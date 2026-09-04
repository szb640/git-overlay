use log::info;

use crate::engine::BaseRepository;

/// Runs `ignore add`: instantiates a [`BaseRepository`] for the current
/// directory, makes sure each pattern is present in the private ignore file
/// outside the managed block, records it in the directory config's ignore
/// patterns, and saves both.
pub fn run_ignore_add(patterns: &[String]) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;

    target.add_ignores(patterns.iter())?;
    for pattern in patterns {
        info!("added ignore={pattern}");
    }

    Ok(())
}

/// Runs `ignore remove`: instantiates a [`BaseRepository`] for the current
/// directory, makes sure each pattern is no longer present in the private
/// ignore file, removes it from the directory config's ignore patterns, and
/// saves both.
pub fn run_ignore_remove(patterns: &[String]) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;

    target.remove_ignores(patterns.iter())?;
    for pattern in patterns {
        info!("removed ignore={pattern}");
    }

    Ok(())
}