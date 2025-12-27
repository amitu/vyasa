use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod add;
mod check;
mod list;
mod mantra;
mod parser;
mod snapshot;
mod stats;
mod status;
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
    /// Show unaccepted mantras/commentaries since last snapshot
    Status {
        /// Path to the repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Accept a mantra/commentary pair into the snapshot
    Add {
        /// Mantra text to accept (optional, shows interactive selection if omitted)
        mantra: Option<String>,
        /// Path to the repository (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Show details about a specific mantra
    Mantra {
        /// The mantra text to look up
        text: String,
        /// Also show where this mantra is referenced
        #[arg(long, short)]
        references: bool,
        /// Path to the repository (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// List all mantras with their acceptance status
    List {
        /// Filter mantras containing this text
        filter: Option<String>,
        /// Only show pending (unaccepted) mantras
        #[arg(long, short)]
        pending: bool,
        /// Path to the repository (defaults to current directory)
        #[arg(long, default_value = ".")]
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
        Commands::Status { path } => status::run(&path),
        Commands::Add { mantra, path } => add::run(&path, mantra),
        Commands::Mantra {
            text,
            references,
            path,
        } => mantra::run(&path, &text, references),
        Commands::List {
            filter,
            pending,
            path,
        } => list::run(&path, filter, pending),
        Commands::Stats { path, buckets } => stats::run(&path, buckets),
        Commands::Values { mantra, key, path } => values::run(&path, mantra, key),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
