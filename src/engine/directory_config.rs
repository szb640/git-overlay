use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Relative location of the configuration file under the directory root.
const CONFIG_PATH: &str = ".git-overlay.yml";

/// Directory-scoped configuration stored as a YAML file inside an overlay
/// directory.
///
/// This is a structured file the tool itself owns and writes. It tracks
/// settings that apply to a single overlay directory, currently a list of
/// managed patterns and a list of ignore patterns.
#[derive(Serialize, Deserialize)]
pub struct DirectoryConfig {
    /// Directory root this configuration was loaded for. Not serialized.
    #[serde(skip)]
    root: PathBuf,
    /// Whether the configuration file existed on disk when loaded. Not
    /// serialized. `false` for a default config whose file has not been
    /// written yet.
    #[serde(skip)]
    exists: bool,
    /// Patterns of files managed in the overlay directory.
    managed_patterns: Vec<String>,
    /// Patterns of files to ignore when scanning the overlay directory.
    ignore_patterns: Vec<String>,
}

impl DirectoryConfig {
    /// A default configuration rooted at `root`, with no managed or ignore
    /// patterns.
    fn default_with(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            exists: false,
            managed_patterns: Vec::new(),
            ignore_patterns: Vec::new(),
        }
    }

    /// Location of the configuration file under `root`.
    pub(crate) fn path(root: &Path) -> PathBuf {
        root.join(CONFIG_PATH)
    }

    /// Loads the configuration for the directory rooted at `root`.
    ///
    /// Returns an empty default configuration if no file exists yet, so the
    /// file does not need to be created by hand.
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = Self::path(root);
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let mut config: Self = serde_yaml::from_str(&content).map_err(|e| {
                    format!("failed to parse {}: {e}", path.display())
                })?;
                config.root = root.to_path_buf();
                config.exists = true;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default_with(root)),
            Err(e) => Err(format!("failed to read {}: {e}", path.display())),
        }
    }

    /// Writes the configuration to disk as YAML, creating the parent
    /// directory if necessary.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path(&self.root());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("failed to create config directory {}: {e}", parent.display())
            })?;
        }
        let content =
            serde_yaml::to_string(self).map_err(|e| format!("failed to serialize config: {e}"))?;
        std::fs::write(&path, content)
            .map_err(|e| format!("failed to write {}: {e}", path.display()))
    }

    /// The directory root this configuration belongs to.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the configuration file existed on disk when this instance was
    /// loaded. A default config that has never been saved reports `false`.
    pub fn exists(&self) -> bool {
        self.exists
    }

    /// The managed patterns for the overlay directory.
    pub fn managed_patterns(&self) -> &[String] {
        &self.managed_patterns
    }

    /// Appends a managed pattern to the directory config, if not already
    /// present.
    pub fn add_managed_pattern(&mut self, pattern: impl Into<String>) {
        let pattern = pattern.into();
        if !self.managed_patterns.iter().any(|p| p == &pattern) {
            self.managed_patterns.push(pattern);
        }
    }

    /// Removes all occurrences of `pattern` from the managed patterns.
    pub fn remove_managed_pattern(&mut self, pattern: impl Into<String>) {
        let pattern = pattern.into();
        self.managed_patterns.retain(|p| p != &pattern);
    }

    /// The ignore patterns for the overlay directory.
    pub fn ignore_patterns(&self) -> &[String] {
        &self.ignore_patterns
    }

    /// Appends an ignore pattern to the directory config, if not already
    /// present.
    pub fn add_ignore_pattern(&mut self, pattern: impl Into<String>) {
        let pattern = pattern.into();
        if !self.ignore_patterns.iter().any(|p| p == &pattern) {
            self.ignore_patterns.push(pattern);
        }
    }

    /// Removes all occurrences of `pattern` from the ignore patterns.
    pub fn remove_ignore_pattern(&mut self, pattern: impl Into<String>) {
        let pattern = pattern.into();
        self.ignore_patterns.retain(|p| p != &pattern);
    }
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self::default_with(&PathBuf::new())
    }
}