use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{
    compare_with_snapshot, compute_hash, create_entry, DefinitionStatus, DefinitionWithStatus,
    Snapshot,
};
use std::io::{self, Write};
use std::path::Path;

pub fn run(path: &Path, mantra_filter: Option<String>) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;
    let mut snapshot = Snapshot::load(&repo_root);

    let statuses = compare_with_snapshot(&repo, &snapshot);

    let pending: Vec<_> = statuses
        .iter()
        .filter(|s| !matches!(s.status, DefinitionStatus::Accepted))
        .collect();

    if pending.is_empty() {
        println!("nothing to add - all mantra/commentary pairs are accepted");
        return Ok(());
    }

    match mantra_filter {
        Some(filter) => add_filtered(&filter, &pending, &mut snapshot, &repo_root),
        None => add_interactive(&pending, &mut snapshot, &repo_root),
    }
}

fn add_filtered(
    filter: &str,
    pending: &[&DefinitionWithStatus],
    snapshot: &mut Snapshot,
    repo_root: &Path,
) -> Result<(), String> {
    // find all pending entries matching this mantra text
    let matches: Vec<_> = pending
        .iter()
        .filter(|s| s.definition.mantra_text == filter)
        .collect();

    if matches.is_empty() {
        // try partial match
        let partial: Vec<_> = pending
            .iter()
            .filter(|s| s.definition.mantra_text.contains(filter))
            .collect();

        if partial.is_empty() {
            return Err(format!("no pending mantra matches '{}'", filter));
        }

        println!("no exact match, did you mean one of these?\n");
        for (i, m) in partial.iter().enumerate() {
            println!(
                "  [{}] {}",
                i + 1,
                truncate(&m.definition.mantra_text, 60)
            );
            println!("      {}:{}", m.definition.file, m.definition.line);
        }
        return Ok(());
    }

    if matches.len() == 1 {
        // single match - accept it
        let entry = matches[0];
        let hash = compute_hash(&entry.definition.commentary);
        snapshot.add_entry(create_entry(&entry.definition, &hash));
        snapshot.save(repo_root)?;
        println!(
            "accepted: ^{}^",
            truncate(&entry.definition.mantra_text, 60)
        );
        println!("          {}:{}", entry.definition.file, entry.definition.line);
        return Ok(());
    }

    // multiple matches - show selection
    println!(
        "this mantra has {} commentaries:\n",
        matches.len()
    );
    for (i, entry) in matches.iter().enumerate() {
        println!("  [{}] {}:{}", i + 1, entry.definition.file, entry.definition.line);
        println!(
            "      \"{}\"",
            truncate(&entry.definition.commentary, 60)
        );
        println!();
    }

    print!("accept which? (1-{}, or 'all'): ", matches.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read input: {}", e))?;

    let input = input.trim();

    if input.eq_ignore_ascii_case("all") {
        for entry in &matches {
            let hash = compute_hash(&entry.definition.commentary);
            snapshot.add_entry(create_entry(&entry.definition, &hash));
        }
        snapshot.save(repo_root)?;
        println!("accepted {} commentaries", matches.len());
        return Ok(());
    }

    if let Ok(num) = input.parse::<usize>() {
        if num >= 1 && num <= matches.len() {
            let entry = matches[num - 1];
            let hash = compute_hash(&entry.definition.commentary);
            snapshot.add_entry(create_entry(&entry.definition, &hash));
            snapshot.save(repo_root)?;
            println!(
                "accepted commentary from {}:{}",
                entry.definition.file, entry.definition.line
            );
            return Ok(());
        }
    }

    Err(format!("invalid selection: {}", input))
}

fn add_interactive(
    pending: &[&DefinitionWithStatus],
    snapshot: &mut Snapshot,
    repo_root: &Path,
) -> Result<(), String> {
    println!("{} pending mantra/commentary pairs:\n", pending.len());

    // show first 10 pending items
    let display_count = pending.len().min(10);
    for (i, entry) in pending.iter().take(display_count).enumerate() {
        println!(
            "  [{}] ^{}^",
            i + 1,
            truncate(&entry.definition.mantra_text, 50)
        );
        println!(
            "      {}:{}",
            entry.definition.file, entry.definition.line
        );
    }

    if pending.len() > 10 {
        println!("\n  ... and {} more", pending.len() - 10);
    }

    println!("\nenter number to accept, or mantra text to filter:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read input: {}", e))?;

    let input = input.trim();

    if input.is_empty() {
        return Ok(());
    }

    // check if it's a number
    if let Ok(num) = input.parse::<usize>() {
        if num >= 1 && num <= display_count {
            let entry = pending[num - 1];
            let hash = compute_hash(&entry.definition.commentary);
            snapshot.add_entry(create_entry(&entry.definition, &hash));
            snapshot.save(repo_root)?;
            println!(
                "\naccepted: ^{}^",
                truncate(&entry.definition.mantra_text, 60)
            );
            return Ok(());
        } else {
            return Err(format!("invalid number: {} (must be 1-{})", num, display_count));
        }
    }

    // treat as filter
    add_filtered(input, pending, snapshot, repo_root)
}

fn truncate(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else if s.contains('\n') {
        format!("{}...", first_line)
    } else {
        first_line.to_string()
    }
}
