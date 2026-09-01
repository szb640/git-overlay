use crate::engine::BaseRepository;
use crate::Settings;

/// Runs the `sync` subcommand: instantiates a [`BaseRepository`] for the
/// current directory and copies its managed (excluded) files into the overlay
/// directory. Only works on initialized repositories.
pub fn run_sync(settings: &Settings) -> Result<(), String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir = std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(root, &dir)?;
    target.sync()
}