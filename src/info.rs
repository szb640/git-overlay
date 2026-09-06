use std::fmt::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::engine::BaseRepository;

/// Runs the `info` subcommand: instantiates a [`BaseRepository`] for the
/// current directory and prints the repository's root, its active private
/// exclude patterns (from `.git/info/exclude`), and the files currently
/// managed by the overlay tool.
///
/// If `json` is true,the result is emitted as a JSON object; otherwisethe
/// same data is printed in a human-readable form. If the repository has not
/// been initialized, only a message stating that is printed (and nothing
/// else) — except in JSON mode, where an object with `initialized: false`
/// is emitted.
pub fn run_info(json: bool) -> Result<String, String> {
    let dir = std::env::current_dir()
        .map_err(|e| format!("failed to get current directory: {e}"))?;

    let target = BaseRepository::new(&dir)?;

    let initialized = target.is_initialized();

    if json {
        return serialize_info(&target, initialized);
    }

    if !initialized {
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
    let ignored = target.list_ignore_patterns();
    writeln!(out, "ignore patterns ({}):", ignored.len())
        .map_err(|e| format!("failed to build output: {e}"))?;
    for pattern in ignored {
        writeln!(out, "  {pattern}")
            .map_err(|e| format!("failed to build output: {e}"))?;
    }

    Ok(out)
}

fn serialize_info(target: &BaseRepository, initialized: bool) -> Result<String,String> {
    #[derive(Serialize)]
    struct InfoData {
        path: Option<PathBuf>,
        initialized: bool,
        #[serde(rename = "exclude_patterns")]
        exclude_patterns: Vec<String>,
        #[serde(rename = "tracked_files")]
        tracked_files: Vec<String>,
        #[serde(rename = "ignore_patterns")]
        ignore_patterns: Vec<String>,
    }

    let data = InfoData {
        path: Some(target.root().to_path_buf()),
        initialized,
        exclude_patterns: if initialized {
            target.list_patterns().to_vec()
        } else {
            Vec::new()
        },
        tracked_files: if initialized {
            target.tracked_files().to_vec()
        } else {
            Vec::new()
        },
        ignore_patterns: if initialized {
            target.list_ignore_patterns().to_vec()
        } else {
            Vec::new()
        },
    };

    let serialized = serde_json::to_string_pretty(&data);
    match serialized {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("failed to serialize info: {e}")),
    }
}