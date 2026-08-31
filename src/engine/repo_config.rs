use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Relative location of the configuration file under the repository root.
const CONFIG_PATH: &str = ".git/overlay.yml";

/// Repository-scoped configuration stored as a YAML file inside the base
/// repository.
///
/// Unlike `.git/info/exclude` (which `ExcludeFile` manages), this is a
/// structured file the tool itself owns and writes. It tracks settings that
/// apply to a single base repository.
#[derive(Serialize, Deserialize)]
pub struct RepositoryConfiguration {
    /// Repository root this configuration was loaded for. Not serialized.
    #[serde(skip)]
    root: PathBuf,
    /// Path to the overlay directory, relative to the repository root.
    overlay_directory: PathBuf,
}

impl RepositoryConfiguration {
    /// A default configuration rooted at `root`, with no overlay directory
    /// set.
    fn default_with(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            overlay_directory: PathBuf::new(),
        }
    }

    /// Location of the configuration file under `root`.
    fn path(root: &Path) -> PathBuf {
        root.join(CONFIG_PATH)
    }

    /// Loads the configuration for the repository rooted at `root`.
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

    /// The repository root this configuration belongs to.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The overlay directory path, relative to the repository root.
    pub fn overlay_directory(&self) -> &Path {
        &self.overlay_directory
    }

    /// Sets the overlay directory path, relative to the repository root.
    pub fn set_overlay_directory(&mut self, path: impl Into<PathBuf>) {
        self.overlay_directory = path.into();
    }
}

impl Default for RepositoryConfiguration {
    fn default() -> Self {
        Self::default_with(&PathBuf::new())
    }
}
