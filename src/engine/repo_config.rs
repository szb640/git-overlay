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
    /// Whether the configuration file existed on disk when loaded. Not
    /// serialized. `false` for a default config whose file has not been
    /// written yet.
    #[serde(skip)]
    exists: bool,
    /// Path to the overlay directory, relative to the repository root.
    overlay_directory: PathBuf,
    /// Files managed by the overlay tool, relative to the repository root.
    managed_files: Vec<String>,
}

impl RepositoryConfiguration {
    /// A default configuration rooted at `root`, with no overlay directory
    /// set.
    fn default_with(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            exists: false,
            overlay_directory: PathBuf::new(),
            managed_files: Vec::new(),
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

    /// Whether the configuration file existed on disk when this instance was
    /// loaded. A default config that has never been saved reports `false`.
    pub fn exists(&self) -> bool {
        self.exists
    }

    /// The files managed by the overlay tool, relative to the repository
    /// root.
    pub fn managed_files(&self) -> &[String] {
        &self.managed_files
    }

    /// Registers a file as managed by the overlay tool, if not already
    /// present.
    pub fn add_managed_file(&mut self, file: impl Into<String>) {
        let file = file.into();
        if !self.managed_files.iter().any(|f| f == &file) {
            self.managed_files.push(file);
        }
    }

    /// Removes a file from the managed files list.
    pub fn remove_managed_file(&mut self, file: impl Into<String>) {
        let file = file.into();
        self.managed_files.retain(|f| f != &file);
    }
}

impl Default for RepositoryConfiguration {
    fn default() -> Self {
        Self::default_with(&PathBuf::new())
    }
}
