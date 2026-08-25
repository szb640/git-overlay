use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "git-overlay", version, about)]
struct Cli {
    /// Increase logging verbosity.
    #[arg(short, long)]
    verbose: bool,

    /// Path to the overlay repository. Falls back to the GIT_OVERLAY_REPO
    /// environment variable when not provided.
    #[arg(long, global = true, env = "GIT_OVERLAY_REPO")]
    overlay_repo: Option<PathBuf>,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Sync repositories
    Sync {
    },
}

fn main() {
    let cli = Cli::parse();

    let default_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();

    match cli.action {
        Action::Sync { } => println!("TODO"),
    }
}