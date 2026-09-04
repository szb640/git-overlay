use std::path::PathBuf;

use clap::Parser;
use log::error;

mod add;
mod engine;
mod info;
mod init;
mod remove;
mod sync;

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "git-overlay", version, about)]
struct Cli {
    /// Increase logging verbosity.
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    /// Sync repositories
    Sync {},

    /// Initialize the current directory as a managed repository, pointing it
    /// at the given overlay directory.
    Init {
        /// Path to the overlay directory
        path: PathBuf,
    },

    /// Add patterns to the private exclude list and save
    Add {
        /// Pattern(s) to add to the private ignore list
        #[arg(required = true)]
        patterns: Vec<String>,
    },

    /// Remove patterns from the private exclude list and save
    Remove {
        /// Pattern(s) to remove from the private ignore list
        #[arg(required = true)]
        patterns: Vec<String>,
    },

    /// Show the active ignore patterns and the tracked (managed) files
    Info {},
}

fn main() {
    let cli = Cli::parse();

    let default_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();

    match cli.action {
        Action::Sync {} => {
            if let Err(e) = sync::run_sync() {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Init { path } => {
            if let Err(e) = init::run_init(&path) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Add { patterns } => {
            if let Err(e) = add::run_add(&patterns) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Remove { patterns } => {
            if let Err(e) = remove::run_remove(&patterns) {
                error!("{e}");
                std::process::exit(1);
            }
        }
        Action::Info {} => match info::run_info() {
            Ok(output) => print!("{output}"),
            Err(e) => {
                error!("{e}");
                std::process::exit(1);
            }
        },
    }
}
