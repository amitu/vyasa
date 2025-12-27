use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod list;
mod mantra;
mod parser;
mod snapshot;
mod stats;
mod status;
mod study;
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
    /// Compare mantras against canon.md
    Status {
        /// Path to the repository (defaults to current directory)
        #[arg(default_value = ".")]
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
    /// List all mantras with their canon status
    List {
        /// Filter mantras containing this text
        filter: Option<String>,
        /// Only show pending (not in canon) mantras
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
    /// Study mantras from configured koshas
    Study {
        /// Kosha alias to study (if omitted, shows stats for all koshas)
        kosha: Option<String>,
        /// Number of mantras to show (default 5)
        #[arg(long, short, default_value = "5")]
        count: usize,
        /// Path to the repository (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { path } => check::run(&path),
        Commands::Status { path } => status::run(&path),
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
        Commands::Study { kosha, count, path } => study::run(&path, kosha, count),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
