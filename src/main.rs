use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod parser;
mod stats;
mod values;

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
    /// Query placeholder values from template mantras
    Values {
        /// Filter by mantra reference, e.g. --mantra="[user: {name}]"
        #[arg(long, short)]
        mantra: Option<String>,
        /// Filter by placeholder key name
        #[arg(long, short)]
        key: Option<String>,
        /// Path to file, folder, or glob pattern (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { path } => check::run(&path),
        Commands::Stats { path, buckets } => stats::run(&path, buckets),
        Commands::Values { mantra, key, path } => values::run(&path, mantra, key),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
