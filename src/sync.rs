use crate::engine::BaseRepository;

/// Runs the `sync` subcommand: instantiates a [`BaseRepository`] for the
/// current directory and copies its managed (excluded) files into the overlay
/// directory. Only works on initialized repositories.
pub fn run_sync() -> Result<(), String> {
    let dir = std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;
    target.sync()
}