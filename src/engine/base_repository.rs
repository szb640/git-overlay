use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::gitignore::GitignoreBuilder;
use log::info;

use crate::engine::exclude::ExcludeFile;
use crate::engine::{OverlayDirectory, RepositoryConfiguration};

/// A repository to be overlaid into the overlay repository.
pub struct BaseRepository {
    /// Absolute path to the repository root on disk.
    repo_root_abs: PathBuf,
    /// This repository's private ignore file (`.git/info/exclude`).
    exclude: ExcludeFile,
    /// This repository's YAML configuration file (`.git-overlay/config.yml`).
    config: RepositoryConfiguration,
    /// This repository's overlay destination (`.git-overlay.yml` rooted in
    /// the overlay directory).
    overlay: OverlayDirectory,
}

impl BaseRepository {
    /// Constructs an overlay target for the given directory, resolving its Git
    /// repository root (top-level directory) and all of its path fields.
    pub fn new(directory: &Path) -> Result<Self, String> {
        let dir = canonicalize(&PathBuf::from(directory))?;

        let info = git_rev_parse(&dir, &["--is-inside-work-tree", "--show-toplevel"])?;
        let (inside, toplevel) = match info.as_slice() {
            [inside, top] => (inside.as_str(), top.as_str()),
            _ => return Err(format!("unexpected git rev-parse output in {}", dir.display())),
        };

        if inside != "true" {
            return Err(format!(
                "current directory {} is not managed by git",
                dir.display()
            ));
        }

        // `dir` sits inside this root, so the Git repository root is the
        // top-level directory reported by Git.
        let repo_root_abs = canonicalize(&PathBuf::from(toplevel))?;

        let exclude = ExcludeFile::load(&repo_root_abs)?;
        let config = RepositoryConfiguration::load(&repo_root_abs)?;
        let overlay = OverlayDirectory::new(&repo_root_abs.join(config.overlay_directory()))?;

        Ok(Self {
            repo_root_abs,
            exclude,
            config,
            overlay,
        })
    }

    /// Returns the patterns in the repository's private ignore file
    /// (`.git/info/exclude`).
    pub fn list_patterns(&self) -> &[String] {
        self.exclude.patterns()
    }

    /// Appends each pattern to the repository's private ignore file
    /// (`.git/info/exclude`) and to the overlay directory's ignore patterns,
    /// writing both to disk in a single save each.
    pub fn add_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        self.ensure_initialized()?;

        // Drop patterns already present in the exclude file, and any
        // duplicates within the batch itself.
        let existing: Vec<String> = self.exclude.patterns().to_vec();
        let mut seen = std::collections::HashSet::new();
        let mut unique: Vec<String> = Vec::new();
        for pattern in patterns.into_iter().map(Into::into) {
            if existing.contains(&pattern) || !seen.insert(pattern.clone()) {
                continue;
            }
            unique.push(pattern);
        }

        for pattern in &unique {
            self.exclude.add(pattern.clone());
        }
        self.exclude.save()?;
        self.overlay.add_patterns(&unique)?;

        // Move + hard-link the files newly matched by these patterns into the
        // overlay.
        let repo_root = &self.repo_root_abs;
        let candidates: Vec<PathBuf> = matching_files(repo_root, &unique)?
            .map(|path| {
                let path = path?;
                path.strip_prefix(repo_root)
                    .map(Path::to_path_buf)
                    .map_err(|e| {
                        format!(
                            "failed to relativize {} to {}: {e}",
                            path.display(),
                            repo_root.display()
                        )
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.sync_added(&candidates)?;
        // Persist the newly managed files so a later `sync` recognizes the
        // moved files as already managed (instead of trying to move them
        // again).
        self.config.save()
    }

    /// Removes all patterns from the repository's private ignore file
    /// (`.git/info/exclude`) and writes it to disk.
    pub fn clear_patterns(&mut self) -> Result<(), String> {
        self.ensure_initialized()?;
        self.exclude.clear();
        self.exclude.save()
    }

    /// Removes each pattern from the repository's private ignore file
    /// (`.git/info/exclude`) and from the overlay directory's ignore
    /// patterns, writing both to disk in a single save each.
    pub fn remove_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        let patterns: Vec<String> = patterns.into_iter().map(Into::into).collect();
        for pattern in &patterns {
            self.exclude.remove(pattern);
        }
        self.exclude.save()?;
        self.overlay.remove_patterns(&patterns)?;

        // Remove from the overlay any managed files that the removed patterns
        // no longer exclude.
        let repo_root = &self.repo_root_abs;
        let excluded: Vec<PathBuf> = self
            .excluded_files()?
            .iter()
            .map(|file| {
                file.strip_prefix(repo_root).map(Path::to_path_buf).map_err(
                    |e| {
                        format!(
                            "failed to relativize {} to {}: {e}",
                            file.display(),
                            repo_root.display()
                        )
                    },
                )
            })
            .collect::<Result<Vec<_>, String>>()?;
        let managed: Vec<String> = self.config.managed_files().to_vec();
        self.sync_removed(&managed, &excluded)
    }

    /// Returns an error if the repository has not been initialized (i.e. its
    /// configuration file does not exist yet). Otherwise refreshes the
    /// in-memory [`OverlayDirectory`] from the configured overlay path.
    fn ensure_initialized(&mut self) -> Result<(), String> {
        if !self.config.exists() {
            return Err(format!(
                "repository {} is not initialized; run init first",
                self.repo_root_abs.display()
            ));
        }
        self.overlay = OverlayDirectory::new(
            &self.repo_root_abs.join(self.config.overlay_directory()),
        )?;
        Ok(())
    }


    /// Initializes the repository: fails if it is already initialized, and
    /// otherwise writes the configuration file with the overlay path set to
    /// `overlay_path` and saves an empty exclude file.
    pub fn init(&mut self, overlay_path: impl Into<PathBuf>) -> Result<(), String> {
        if self.config.exists() {
            return Err(format!(
                "repository {} is already initialized with overlay path {}",
                self.repo_root_abs.display(),
                self.config.overlay_directory().display()
            ));
        }
        self.config.set_overlay_directory(overlay_path.into());
        self.config.save()?;
        self.overlay = OverlayDirectory::new(
            &self.repo_root_abs.join(self.config.overlay_directory()),
        )?;
        self.exclude.save()?;

        // Bring any overlay files into the repository (as hard links) and
        // make them private to this clone by folding them into the exclude
        // file, so they do not show up as untracked in `git status`.
        self.sync()
    }

    /// Moves each candidate file that is excluded but not yet managed into the
    /// overlay directory and hard links it back into the repository, then
    /// registers it in the config. Skips files already recorded as managed.
    fn sync_added(&mut self, candidates: &[PathBuf]) -> Result<(), String> {
        let repo_root = &self.repo_root_abs;
        let overlay_dir = self.overlay.root();
        let managed: Vec<String> = self.config.managed_files().to_vec();

        for rel in candidates {
            let rel_str = rel.to_string_lossy().into_owned();
            if managed.contains(&rel_str) {
                continue;
            }

            let file = repo_root.join(rel);
            let dest = overlay_dir.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("failed to create {}: {e}", parent.display())
                })?;
            }
            std::fs::rename(&file, &dest).map_err(|e| {
                format!("failed to move {} to {}: {e}", file.display(), dest.display())
            })?;
            std::fs::hard_link(&dest, &file).map_err(|e| {
                format!("failed to link {} back to {}: {e}", dest.display(), file.display())
            })?;
            info!("moved {} to {}", file.display(), dest.display());
            self.config.add_managed_file(rel_str);
        }

        Ok(())
    }

    /// Removes from the overlay directory any file that is listed as managed
    /// in the config but no longer among the currently excluded files, and
    /// unregisters it from the config.
    fn sync_removed(
        &mut self,
        managed: &[String],
        excluded: &[PathBuf],
    ) -> Result<(), String> {
        let overlay_dir = self.overlay.root();

        for rel in managed {
            let rel_path = Path::new(rel);
            if excluded.iter().any(|e| e == rel_path) {
                continue;
            }

            let dest = overlay_dir.join(&rel_path);
            std::fs::remove_file(&dest).map_err(|e| {
                format!("failed to remove {}: {e}", dest.display())
            })?;
            info!("removed {} from overlay", dest.display());
            self.config.remove_managed_file(rel.clone());
        }

        Ok(())
    }

    /// Folds any ignore patterns present in the overlay config (but not yet in
    /// the exclude file) into the repository's private ignore file, keeping
    /// both sources in sync.
    fn sync_overlay_patterns(&mut self) -> Result<(), String> {
        let exclude_patterns: Vec<String> = self.exclude.patterns().to_vec();
        for pattern in self.overlay.list_patterns() {
            if exclude_patterns.contains(pattern) {
                continue;
            }
            self.exclude.add(pattern.clone());
            info!("added exclude_pattern={pattern} from overlay");
        }
        self.exclude.save()
    }

    /// Hard links any overlay files that are not present in the repository
    /// back into it, at the same relative path.
    fn sync_overlay_files(&mut self) -> Result<(), String> {
        let overlay_dir = self.overlay.root();
        let repo_root = &self.repo_root_abs;

        for file in self.overlay.files()? {
            let rel = file.strip_prefix(overlay_dir).map_err(|_| {
                format!("failed to relativize {} to {}", file.display(), overlay_dir.display())
            })?;
            let dest = repo_root.join(rel);
            if dest.exists() {
                continue;
            }
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("failed to create {}: {e}", parent.display())
                })?;
            }
            std::fs::hard_link(&file, &dest).map_err(|e| {
                format!("failed to link {} to {}: {e}", file.display(), dest.display())
            })?;
            info!("linked {} to {}", file.display(), dest.display());
            // A file pulled from the overlay is managed; record it so a later
            // `remove`/`sync` knows to drop it from the overlay again.
            self.config.add_managed_file(rel.to_string_lossy().into_owned());
        }

        Ok(())
    }

    /// Syncs the overlay with the repository's current managed (excluded)
    /// files. For each file currently excluded but not yet managed, moves it
    /// into the overlay directory and hard links it back into the repository,
    /// and registers it in the config. For each file managed in the config but
    /// no longer excluded, removes it from the overlay directory and
    /// unregisters it from the config. Only works on initialized repositories.
    pub fn sync(&mut self) -> Result<(), String> {
        self.ensure_initialized()?;

        let repo_root = &self.repo_root_abs;

        // Relativized paths of the files currently excluded in the repository.
        let mut excluded: Vec<PathBuf> = Vec::new();
        for file in self.excluded_files()? {
            let rel = file.strip_prefix(repo_root).map_err(|_| {
                format!("failed to relativize {} to {}", file.display(), repo_root.display())
            })?;
            excluded.push(rel.to_path_buf());
        }

        // In-memory snapshot of the managed list for membership checks, since
        // it may be mutated below.
        let managed: Vec<String> = self.config.managed_files().to_vec();

        // Move + hard-link files that are excluded but not yet managed.
        self.sync_added(&excluded)?;

        // Remove overlay files that are managed but no longer excluded.
        self.sync_removed(&managed, &excluded)?;

        self.overlay.fix_patterns()?;

        // Fold any ignore patterns present in the overlay config (but not yet
        // in the exclude file) into the exclude file.
        self.sync_overlay_patterns()?;

        // Bring any overlay files that are missing from the repository back in
        // via hard links.
        self.sync_overlay_files()?;

        self.config.save()
    }

    /// Returns the absolute paths of all files in the repository that match
    /// the exclude patterns managed by this target's private ignore file.
    pub fn excluded_files(&self) -> Result<Vec<PathBuf>, String> {
        let files = matching_files(&self.repo_root_abs, self.exclude.patterns())?;
        files.collect()
    }
}

/// Walks `root` and returns an iterator over the absolute paths of files that
/// match any of `patterns` (interpreted as gitignore patterns).
///
/// Each item is `Ok(path)` for a matching file, or `Err` if walking fails.
/// Errors while building the matcher (e.g. an invalid pattern) are returned
/// eagerly.
fn matching_files<'a>(
    root: &'a Path,
    patterns: impl IntoIterator<Item = impl AsRef<str>> + 'a,
) -> Result<impl Iterator<Item = Result<PathBuf, String>> + 'a, String> {
    let mut builder = GitignoreBuilder::new(root);
    for pattern in patterns {
        builder
            .add_line(None, pattern.as_ref())
            .map_err(|e| format!("invalid exclude pattern {:?}: {e}", pattern.as_ref()))?;
    }
    let matcher = builder
        .build()
        .map_err(|e| format!("failed to build exclude matcher: {e}"))?;

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(false)
        .git_exclude(false)
        .git_global(false)
        .parents(false)
        .build();

    Ok(walker.filter_map(move |entry| {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => return Some(Err(format!("failed to walk repository: {e}"))),
        };
        let is_dir = entry.file_type().map_or(false, |t| t.is_dir());
        if is_dir {
            return None;
        }
        if matcher.matched(entry.path(), false).is_ignore() {
            Some(Ok(entry.path().to_path_buf()))
        } else {
            None
        }
    }))
}

/// Canonicalizes a path, erroring if it does not exist.
fn canonicalize(path: &PathBuf) -> Result<PathBuf, String> {
    std::fs::canonicalize(path)
        .map_err(|e| format!("failed to resolve path {}: {e}", path.display()))
}

/// Runs `git rev-parse` with the given arguments in `dir`, returning the
/// trimmed stdout lines on success.
fn git_rev_parse(dir: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("rev-parse")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git in {}: {e}", dir.display()))?;

    if !out.status.success() {
        return Err(format!(
            "git rev-parse {} failed in {}: {}",
            args.join(" "),
            dir.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}
