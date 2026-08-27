use log::info;

use crate::overlay::OverlayTarget;
use crate::Settings;

/// Runs the `info` subcommand: instantiates an [`OverlayTarget`] for the
/// current directory and logs the patterns in its private ignore file
/// (`.git/info/exclude`).
pub fn run_info(settings: &Settings) -> Result<(), String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir = std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let target = OverlayTarget::new(root, &dir)?;

    for pattern in target.exclude().patterns() {
        info!("exclude_pattern={pattern}");
    }

    Ok(())
}