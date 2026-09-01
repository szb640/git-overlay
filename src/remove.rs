use log::info;

use crate::engine::BaseRepository;

/// Runs the `remove` subcommand: instantiates an [`BaseRepository`] for the
/// current directory, removes the given patterns from its private ignore
/// list, and saves the file.
pub fn run_remove(patterns: &[String]) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;

    target.remove_patterns(patterns.iter())?;
    for pattern in patterns {
        info!("removed exclude_pattern={pattern}");
    }

    Ok(())
}