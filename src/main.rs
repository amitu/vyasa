use clap::Parser;
use std::path::PathBuf;

mod check;
mod mantra;
mod parser;
mod stats;

#[derive(Parser)]
#[command(name = "vyasa")]
#[command(about = "A tool to organize and curate knowledge through mantras")]
struct Cli {
    /// Mantra text to search for (omit to run check + stats)
    mantra: Option<String>,

    /// Path to the repository (defaults to current directory)
    #[arg(long, short, default_value = ".")]
    path: PathBuf,

    /// Show anusrits when searching for a mantra
    #[arg(long, short)]
    anusrits: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.mantra {
        Some(text) => mantra::run(&cli.path, &text, cli.anusrits),
        None => run_check_and_stats(&cli.path),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_check_and_stats(path: &PathBuf) -> Result<(), String> {
    // run check first
    let check_result = check::run(path);

    // always show stats after check output
    println!();
    stats::run(path, 10)?;

    // return check result (may be error)
    check_result
}
