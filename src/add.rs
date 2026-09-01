use log::info;

use crate::engine::BaseRepository;
use crate::Settings;

/// Runs the `add` subcommand: instantiates an [`BaseRepository`] for the
/// current directory, appends the given patterns to its private ignore list,
/// and saves the file.
pub fn run_add(settings: &Settings, patterns: &[String]) -> Result<(), String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(root, &dir)?;

    target.add_patterns(patterns.iter())?;
    for pattern in patterns {
        info!("added exclude_pattern={pattern}");
    }

    Ok(())
}