use log::info;

use crate::overlay::OverlayTarget;
use crate::Settings;

/// Runs the `reset` subcommand: deletes every file that matched the exclude
/// patterns, then clears the exclude patterns themselves and saves.
pub fn run_reset(settings: &Settings) -> Result<(), String> {
    let root = settings.repository_root.as_ref().unwrap();
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let mut target = OverlayTarget::new(root, &dir)?;

    for path in target.excluded_files()? {
        std::fs::remove_file(&path)
            .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
        info!("removed file={}", path.display());
    }

    target.exclude_mut().clear();
    target.exclude_mut().save()?;

    Ok(())
}