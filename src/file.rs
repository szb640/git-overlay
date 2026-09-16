use std::path::{Component, Path, PathBuf};

use log::info;

use crate::engine::BaseRepository;

/// Runs the `file add` subcommand: translates each given path into a
/// root-anchored exclude pattern that matches only that exact file in the
/// repository, then adds and manages it.
pub fn run_file_add(paths: &[PathBuf]) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;
    let mut target = BaseRepository::new(&cwd)?;

    let patterns = paths_to_patterns(target.root(), &cwd, paths)?;
    target.add_patterns(&patterns)?;
    for pattern in &patterns {
        info!("added exclude_pattern={pattern}");
    }

    Ok(())
}

/// Runs the `file remove` subcommand: translates each given path into the
/// same root-anchored exclude pattern used by [`run_file_add`] and removes
/// it, unmanaging the file.
pub fn run_file_remove(paths: &[PathBuf]) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;
    let mut target = BaseRepository::new(&cwd)?;

    let patterns = paths_to_patterns(target.root(), &cwd, paths)?;
    target.remove_patterns(&patterns)?;
    for pattern in &patterns {
        info!("removed exclude_pattern={pattern}");
    }

    Ok(())
}

/// Converts each path into a root-anchored gitignore pattern (`/rel/path`)
/// that matches only the file at that path relative to the repository root,
/// rather than any file of that name elsewhere in the tree.
///
/// Relative paths are resolved against the current working directory, and the
/// result is required to stay inside the repository.
fn paths_to_patterns(
    repo_root: &Path,
    cwd: &Path,
    paths: &[PathBuf],
) -> Result<Vec<String>, String> {
    let repo_root = repo_root
        .canonicalize()
        .map_err(|e| format!("failed to resolve repository root {}: {e}", repo_root.display()))?;
    let cwd = cwd
        .canonicalize()
        .map_err(|e| format!("failed to resolve current directory {}: {e}", cwd.display()))?;

    paths
        .iter()
        .map(|input| {
            let abs = if input.is_absolute() {
                input.clone()
            } else {
                cwd.join(input)
            };
            let rel = abs.strip_prefix(&repo_root).map_err(|_| {
                format!(
                    "path {} is outside the repository {}",
                    input.display(),
                    repo_root.display()
                )
            })?;
            let rel = normalize_rel(rel);
            if rel.is_empty() {
                return Err(format!(
                    "path {} is the repository root, not a file",
                    input.display()
                ));
            }
            Ok(format!("/{rel}"))
        })
        .collect()
}

/// Lexically normalizes a repository-relative path, removing `.` components
/// and resolving `..` components. Because the path is already known to lie
/// inside the repository root, no `..` can escape above it, so normalizing is
/// always safe.
fn normalize_rel(path: &Path) -> String {
    let mut norm: Vec<String> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                norm.pop();
            }
            other => norm.push(other.as_os_str().to_string_lossy().into_owned()),
        }
    }
    norm.join("/")
}