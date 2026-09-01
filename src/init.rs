use std::path::Path;

use crate::engine::BaseRepository;

/// Runs the `init` subcommand: instantiates an [`BaseRepository`] for the
/// current directory and records the given overlay path in its configuration.
pub fn run_init(overlay_path: &Path) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;
    target.init(overlay_path)
}