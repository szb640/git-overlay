use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::gitignore::GitignoreBuilder;

use crate::engine::exclude::ExcludeFile;
use crate::engine::RepositoryConfiguration;

/// A repository to be overlaid into the overlay repository.
pub struct BaseRepository {
    /// Relative path to the repository root (relative to `repository_root`).
    repo_root: PathBuf,
    /// Absolute path to the repository root on disk.
    repo_root_abs: PathBuf,
    /// Relative path inside the Git folder (relative to the repository root).
    directory: PathBuf,
    /// This repository's private ignore file (`.git/info/exclude`).
    exclude: ExcludeFile,
    /// This repository's YAML configuration file (`.git-overlay/config.yml`).
    config: RepositoryConfiguration,
}

impl BaseRepository {
    /// Constructs an overlay target for the current directory inside
    /// `repository_root`, computing all of its path fields.
    pub fn new(repository_root: &Path, directory: &Path) -> Result<Self, String> {
        let root = canonicalize(&PathBuf::from(repository_root))?;
        let dir = canonicalize(&PathBuf::from(directory))?;

        // Reject directories outside `repository_root` before spawning Git.
        if !path_contains(&root, &dir) {
            return Err(format!(
                "current directory {} is not inside repository_root {}",
                dir.display(),
                root.display()
            ));
        }

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

        
        // Resolve the Git root first: since `dir` sits inside this root, and
        // we check below that the root is inside `repository_root`, `dir` is
        // guaranteed to be inside it too, so no separate check is needed.
        let repo_root_abs = canonicalize(&PathBuf::from(toplevel))?;

        let repo_root = repo_root_abs.strip_prefix(&root).map_err(|_| {
            format!(
                "Git repository root {} is outside repository_root {}",
                repo_root_abs.display(),
                root.display()
            )
        })?;
        let directory = dir.strip_prefix(&repo_root_abs).map_err(|_| {
            format!(
                "current directory {} is outside Git repository root {}",
                dir.display(),
                repo_root_abs.display()
            )
        })?;

        let exclude = ExcludeFile::load(&repo_root_abs)?;
        let config = RepositoryConfiguration::load(&repo_root_abs)?;

        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            repo_root_abs,
            directory: directory.to_path_buf(),
            exclude,
            config,
        })
    }

    /// Relative path to the repository root (relative to `repository_root`).
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Absolute path to the repository root.
    pub fn repo_root_abs(&self) -> &Path {
        &self.repo_root_abs
    }

    /// Relative path inside the Git folder.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Returns the patterns in the repository's private ignore file
    /// (`.git/info/exclude`).
    pub fn list_patterns(&self) -> &[String] {
        self.exclude.patterns()
    }

    /// Appends each pattern to the repository's private ignore file
    /// (`.git/info/exclude`) and writes it to disk in a single save.
    pub fn add_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        for pattern in patterns {
            self.exclude.add(pattern);
        }
        self.exclude.save()
    }

    /// Removes all patterns from the repository's private ignore file
    /// (`.git/info/exclude`) and writes it to disk.
    pub fn clear_patterns(&mut self) -> Result<(), String> {
        self.ensure_initialized()?;
        self.exclude.clear();
        self.exclude.save()
    }

    /// Returns an error if the repository has not been initialized (i.e. its
    /// configuration file does not exist yet).
    fn ensure_initialized(&self) -> Result<(), String> {
        if self.config.exists() {
            Ok(())
        } else {
            Err(format!(
                "repository {} is not initialized; run init first",
                self.repo_root_abs.display()
            ))
        }
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
        self.exclude.save()
    }

    /// Returns the absolute paths of all files in the repository that match
    /// the exclude patterns managed by this target's private ignore file.
    pub fn excluded_files(&self) -> Result<Vec<PathBuf>, String> {
        let mut builder = GitignoreBuilder::new(&self.repo_root_abs);
        for pattern in self.exclude.patterns() {
            builder
                .add_line(None, pattern)
                .map_err(|e| format!("invalid exclude pattern {pattern:?}: {e}"))?;
        }
        let matcher = builder
            .build()
            .map_err(|e| format!("failed to build exclude matcher: {e}"))?;

        let mut matched = Vec::new();
        let walker = WalkBuilder::new(&self.repo_root_abs)
            .hidden(false)
            .git_ignore(false)
            .git_exclude(false)
            .git_global(false)
            .parents(false)
            .build();

        for entry in walker {
            let entry = entry.map_err(|e| format!("failed to walk repository: {e}"))?;
            let is_dir = entry.file_type().map_or(false, |t| t.is_dir());
            if is_dir {
                continue;
            }
            if matcher.matched(entry.path(), false).is_ignore() {
                matched.push(entry.path().to_path_buf());
            }
        }

        Ok(matched)
    }
}

/// Canonicalizes a path, erroring if it does not exist.
fn canonicalize(path: &PathBuf) -> Result<PathBuf, String> {
    std::fs::canonicalize(path)
        .map_err(|e| format!("failed to resolve path {}: {e}", path.display()))
}

/// Returns true if `directory` is under (or equal to) `root`.
fn path_contains(root: &Path, directory: &Path) -> bool {
    directory.starts_with(root)
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
