use std::path::PathBuf;

use log::info;

use crate::overlay::OverlayTarget;
use crate::Settings;

/// Returns the absolute, canonicalized current working directory.
fn current_dir() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))
}

/// Imports the current directory into the overlay.
pub fn import_path(settings: &Settings) -> Result<OverlayTarget, String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir = current_dir()?;

    OverlayTarget::new(root, &dir)
}

/// Imports the current directory into the overlay.
pub fn run_import(settings: &Settings) -> Result<(), String> {
    let target = import_path(settings)?;

    info!(
        "importing repo_root={} repo_root_abs={} directory={}",
        target.repo_root().display(),
        target.repo_root_abs().display(),
        target.directory().display()
    );

    Ok(())
}