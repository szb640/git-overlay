use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use log::warn;
use serde::Deserialize;

/// Path to the configuration file, relative to the user config directory.
const CONFIG_FILE: &str = "config.yml";

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "git-overlay", version, about)]
struct Cli {
    /// Increase logging verbosity.
    #[arg(short, long)]
    verbose: bool,

    /// Path to the overlay repository. Falls back to the GIT_OVERLAY_PATH
    /// environment variable, then to the config file.
    #[arg(long, global = true, env = "GIT_OVERLAY_PATH")]
    overlay_path: Option<PathBuf>,

    /// Root directory containing the repositories to sync. Falls back to the
    /// GIT_OVERLAY_REPOSITORY_ROOT environment variable, then to the config
    /// file.
    #[arg(long, global = true, env = "GIT_OVERLAY_REPOSITORY_ROOT")]
    repository_root: Option<PathBuf>,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Sync repositories
    Sync {
    },
}

/// The parsed contents of the config file. Unknown fields are ignored so the
/// file can grow over time without breaking older binaries.
#[derive(Deserialize, Default)]
struct Config {
    #[serde(alias = "overlay_repo")]
    overlay_path: Option<String>,
    repository_root: Option<String>,
}

/// Returns the path to the config file, e.g.
/// `~/.config/git-overlay/config.yml` on Unix (or the equivalent config
/// directory on other platforms, e.g. `%APPDATA%` on Windows).
fn config_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?;
    Some(dir.join("git-overlay").join(CONFIG_FILE))
}

/// Reads `overlay_path` from the config file, if present and valid.
fn overlay_path_from_config() -> Option<PathBuf> {
    let path = config_path()?;
    load_config(&path).and_then(|c| c.overlay_path.map(PathBuf::from))
}

/// Reads `repository_root` from the config file, if present and valid.
fn repository_root_from_config() -> Option<PathBuf> {
    let path = config_path()?;
    load_config(&path).and_then(|c| c.repository_root.map(PathBuf::from))
}

fn load_config(path: &Path) -> Option<Config> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            warn!("could not read config {}: {e}", path.display());
            return None;
        }
    };

    match serde_yml::from_str::<Config>(&content) {
        Ok(config) => Some(config),
        Err(e) => {
            warn!("invalid config {}: {e}", path.display());
            None
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let default_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();

    // Precedence: CLI flag > GIT_OVERLAY_PATH env var (both handled by clap) >
    // config file.
    let overlay_path = cli.overlay_path.or_else(overlay_path_from_config);

    let repository_root = cli
        .repository_root
        .or_else(repository_root_from_config);

    match cli.action {
        Action::Sync {} => println!(
            "TODO: overlay_path={}, repository_root={}",
            overlay_path.map_or("none".into(), |p| p.to_string_lossy().into_owned()),
            repository_root.map_or("none".into(), |p| p.to_string_lossy().into_owned())
        ),
    }
}
