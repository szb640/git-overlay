use log::info;

use crate::engine::BaseRepository;

/// Runs the `add` subcommand: instantiates an [`BaseRepository`] for the
/// current directory, appends the given patterns to its private ignore list,
/// and saves the file.
pub fn run_add(patterns: &[String]) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;

    target.add_patterns(patterns.iter())?;
    for pattern in patterns {
        info!("added exclude_pattern={pattern}");
    }

    Ok(())
}