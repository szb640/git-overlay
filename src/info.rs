use std::fmt::Write;

use crate::engine::BaseRepository;

/// Runs the `info` subcommand: instantiates a [`BaseRepository`] for the
/// current directory and prints the repository's root, its active private
/// exclude patterns (from `.git/info/exclude`), and the files currently
/// managed by the overlay tool.
///
/// If the repository has not been initialized, only a message stating that
/// is printed (and nothing else).
pub fn run_info() -> Result<String, String> {
    let dir =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

    let target = BaseRepository::new(&dir)?;

    if !target.is_initialized() {
        return Ok(format!(
            "repository {} is not initialized; run init first\n",
            target.root().display()
        ));
    }

    let mut out = String::new();
    writeln!(out, "repository: {}", target.root().display())
        .map_err(|e| format!("failed to build output: {e}"))?;

    let patterns = target.list_patterns();
    writeln!(out, "exclude patterns ({}):", patterns.len())
        .map_err(|e| format!("failed to build output: {e}"))?;
    for pattern in patterns {
        writeln!(out, "  {pattern}")
            .map_err(|e| format!("failed to build output: {e}"))?;
    }

    let files = target.tracked_files();
    writeln!(out, "tracked files ({}):", files.len())
        .map_err(|e| format!("failed to build output: {e}"))?;
    for file in files {
        writeln!(out, "  {file}").map_err(|e| format!("failed to build output: {e}"))?;
    }

    Ok(out)
}