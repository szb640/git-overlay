use std::path::{Path, PathBuf};

use crate::engine::DirectoryConfig;

/// State associated with a `.git-overlay` destination.
pub struct OverlayDirectory {
    /// Absolute path to the overlay directory root on disk.
    root: PathBuf,
    /// This directory's YAML configuration file (`.git-overlay.yml`).
    config: DirectoryConfig,
}

impl OverlayDirectory {
    /// Constructs an overlay directory wrapper for the given root folder,
    /// loading its configuration if one exists. The folder need not exist
    /// yet; the path is stored as given (made absolute if possible).
    pub fn new(root: &Path) -> Result<Self, String> {
        let root = match std::path::absolute(root) {
            Ok(abs) => abs,
            Err(_) => root.to_path_buf(),
        };
        let config = DirectoryConfig::load(&root)?;
        Ok(Self { root, config })
    }

    /// Absolute path to the overlay directory root on disk.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the ignore patterns configured for the directory.
    pub fn list_patterns(&self) -> &[String] {
        self.config.ignore_patterns()
    }

    /// Returns the absolute paths of all files in the overlay directory,
    /// excluding the directory's own configuration file (`.git-overlay.yml`).
    pub fn files(&self) -> Result<Vec<PathBuf>, String> {
        let config_path = self.root.join(DirectoryConfig::path(&self.root));
        let mut files = Vec::new();

        let walker = ignore::WalkBuilder::new(&self.root).build();
        for entry in walker {
            let entry = entry
                .map_err(|e| format!("failed to walk overlay directory: {e}"))?;
            let is_dir = entry.file_type().map_or(false, |t| t.is_dir());
            if is_dir {
                continue;
            }
            if entry.path() == config_path {
                continue;
            }
            files.push(entry.path().to_path_buf());
        }

        Ok(files)
    }

    /// Appends each pattern to the directory's ignore patterns and writes the
    /// config to disk in a single save.
    pub fn add_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        for pattern in patterns {
            self.config.add_ignore_pattern(pattern);
        }
        self.config.save()
    }

    /// Removes each pattern from the directory's ignore patterns and writes
    /// the config to disk in a single save.
    pub fn remove_patterns(
        &mut self,
        patterns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        for pattern in patterns {
            self.config.remove_ignore_pattern(pattern);
        }
        self.config.save()
    }

    /// Finds every file in the overlay directory that is not matched by any
    /// of the configured ignore patterns and adds it as an explicit relative
    /// path to the ignore patterns, then writes the config to disk in a
    /// single save.
    ///
    /// This "pins" stray files that were added by hand (or by a pattern that
    /// no longer covers them) so they are acknowledged as managed, rather
    /// than silently remaining outside the ignore rules.
    pub fn fix_patterns(&mut self) -> Result<(), String> {
        let mut builder = ignore::gitignore::GitignoreBuilder::new(&self.root);
        for pattern in self.config.ignore_patterns() {
            builder
                .add_line(None, pattern)
                .map_err(|e| format!("invalid ignore pattern {pattern:?}: {e}"))?;
        }
        let matcher = builder
            .build()
            .map_err(|e| format!("failed to build overlay ignore matcher: {e}"))?;

        for file in self.files()? {
            if matcher.matched(&file, false).is_ignore() {
                continue;
            }
            let rel = file
                .strip_prefix(&self.root)
                .map_err(|e| {
                    format!(
                        "failed to relativize {} to {}: {e}",
                        file.display(),
                        self.root.display()
                    )
                })?
                .to_string_lossy()
                .into_owned();
            self.config.add_ignore_pattern(rel);
        }

        self.config.save()
    }
}