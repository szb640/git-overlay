use crate::engine::{BaseRepository, ForceSide};

/// Runs the `sync` subcommand: instantiates a [`BaseRepository`] for the
/// current directory and copies its managed (excluded) files into the overlay
/// directory. Only works on initialized repositories.
///
/// `force` optionally resolves conflicts where a file exists in both the
/// repository and the overlay with different contents: `ForceSide::Repository`
/// keeps the repository copy, `ForceSide::Overlay` keeps the overlay copy.
pub fn run_sync(force: Option<ForceSide>) -> Result<(), String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = BaseRepository::new(&dir)?;
    target.sync(force)
}
