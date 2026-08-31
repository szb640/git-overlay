use log::info;

use crate::engine::BaseRepository;
use crate::Settings;

/// Runs the `info` subcommand: instantiates an [`BaseRepository`] for the
/// current directory and logs the patterns in its private ignore file
/// (`.git/info/exclude`).
pub fn run_info(settings: &Settings) -> Result<(), String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir = std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let target = BaseRepository::new(root, &dir)?;

    for pattern in target.exclude().patterns() {
        info!("exclude_pattern={pattern}");
    }

    for path in target.excluded_files()? {
        info!("overlay_file={}", path.display());
    }

    Ok(())
}
