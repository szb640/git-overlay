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

    for pattern in patterns {
        target.exclude_mut().add(pattern.clone());
        info!("added exclude_pattern={pattern}");
    }

    target.exclude_mut().save()?;
    Ok(())
}