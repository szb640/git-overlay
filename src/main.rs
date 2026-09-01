use std::path::PathBuf;

use clap::Parser;
use figment::providers::{Env, Format, Serialized, Yaml};
use figment::Figment;
use log::error;
use serde::{Deserialize, Serialize};

mod add;
mod engine;
mod import;
mod info;
mod reset;
mod sync;

/// Path to the configuration file, relative to the user config directory.
const CONFIG_FILE: &str = "config.yml";

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "git-overlay", version, about)]
struct Cli {
    /// Increase logging verbosity.
    #[arg(short, long)]
    verbose: bool,

    /// Path to the overlay repository. Falls back to the GIT_OVERLAY_OVERLAY_PATH
    /// environment variable, then to the config file.
    #[arg(long, global = true)]
    overlay_path: Option<PathBuf>,

    /// Root directory containing the repositories to sync. Falls back to the
    /// GIT_OVERLAY_REPOSITORY_ROOT environment variable, then to the config
    /// file.
    #[arg(long, global = true)]
    repository_root: Option<PathBuf>,

    #[command(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    /// Sync repositories
    Sync {},

    /// Show info about a repository, including its private exclude patterns
    Info {},

    /// Add patterns to the private exclude list and save
    Add {
        /// Pattern(s) to add to the private ignore list
        #[arg(required = true)]
        patterns: Vec<String>,
    },

    /// Delete files matching the exclude patterns and clear them
    Reset {},

    /// Import a repository from a local directory into the overlay
    Import {
        /// Symlink the directory into the overlay rather than copying it
        #[arg(short, long)]
        link: bool,
    },
}

/// A single configuration setting that may come from the config file, the
/// GIT_OVERLAY_* environment, or the CLI. Each source only contributes the
/// fields it actually sets (None fields are skipped on serialize), so a
/// lower-precedence source is never clobbered by an unset higher one.
#[derive(Serialize, Deserialize, Default)]
struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    overlay_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository_root: Option<PathBuf>,
}

/// Returns the path to the config file, e.g.
/// `~/.config/git-overlay/config.yml` on Unix (or the equivalent config
/// directory on other platforms, e.g. `%APPDATA%` on Windows).
fn config_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?;
    Some(dir.join("git-overlay").join(CONFIG_FILE))
}

/// Builds the layered configuration and resolves the effective settings.
///
/// Precedence, lowest to highest: built-in defaults, config file, the
/// GIT_OVERLAY_* environment variables, then CLI flags.
///
/// Errors if the configuration cannot be resolved, or if `overlay_path` or
/// `repository_root` were never set by any source.
fn settings(cli: &Cli) -> Result<Settings, String> {
    let mut figment = Figment::new().merge(Serialized::defaults(&Settings::default()));

    if let Some(path) = config_path() {
        figment = figment.merge(Yaml::file(path));
    }

    figment = figment.merge(Env::prefixed("GIT_OVERLAY_"));
    figment = figment.merge(Serialized::defaults(&Settings {
        overlay_path: cli.overlay_path.clone(),
        repository_root: cli.repository_root.clone(),
    }));

    let settings: Settings = figment
        .extract()
        .map_err(|e| format!("failed to resolve configuration: {e}"))?;

    if settings.overlay_path.is_none() {
        return Err("overlay_path is not defined (set it via --overlay-path, the \
GIT_OVERLAY_OVERLAY_PATH environment variable, or the config file)"
            .into());
    }
    if settings.repository_root.is_none() {
        return Err("repository_root is not defined (set it via --repository-root, the \
GIT_OVERLAY_REPOSITORY_ROOT environment variable, or the config file)"
            .into());
    }

    Ok(settings)
}

fn main() {
    let cli = Cli::parse();

    let default_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();

    let settings = match settings(&cli) {
        Ok(settings) => settings,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };

    match cli.action {
        Action::Sync {} => {
            if let Err(e) = sync::run_sync(&settings) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Info {} => {
            if let Err(e) = info::run_info(&settings) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Reset {} => {
            if let Err(e) = reset::run_reset(&settings) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Add { patterns } => {
            if let Err(e) = add::run_add(&settings, &patterns) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Import { link: _ } => {
            if let Err(e) = import::run_import(&settings) {
                error!("{e}");
                std::process::exit(1);
            }
        }
    }
}
