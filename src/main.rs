use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod parser;
mod stats;

#[derive(Parser)]
#[command(name = "vyasa")]
#[command(about = "A tool to organize and curate knowledge through mantras")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if all mantras have at least one explanation
    Check {
        /// Path to the repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show statistics about mantras in the repository
    Stats {
        /// Path to the repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Maximum number of buckets for reference histogram (0 for no bucketing)
        #[arg(long, default_value = "10")]
        buckets: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { path } => check::run(&path),
        Commands::Stats { path, buckets } => stats::run(&path, buckets),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
