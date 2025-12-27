use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_snapshot, DefinitionStatus, Snapshot};
use std::path::Path;

pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;
    let snapshot = Snapshot::load(&repo_root);

    let statuses = compare_with_snapshot(&repo, &snapshot);

    let new_entries: Vec<_> = statuses
        .iter()
        .filter(|s| matches!(s.status, DefinitionStatus::New))
        .collect();

    let changed_entries: Vec<_> = statuses
        .iter()
        .filter(|s| matches!(s.status, DefinitionStatus::Changed { .. }))
        .collect();

    let accepted_count = statuses
        .iter()
        .filter(|s| matches!(s.status, DefinitionStatus::Accepted))
        .count();

    if new_entries.is_empty() && changed_entries.is_empty() {
        println!(
            "all {} mantra/commentary pairs are accepted",
            statuses.len()
        );
        return Ok(());
    }

    // show new entries
    if !new_entries.is_empty() {
        println!(
            "{} new mantra/commentary pairs since last snapshot:\n",
            new_entries.len()
        );
        for entry in &new_entries {
            println!("  {}:{}", entry.definition.file, entry.definition.line);
            println!("    ^{}^", truncate(&entry.definition.mantra_text, 60));
            if !entry.definition.commentary.is_empty() {
                println!("    \"{}\"", truncate(&entry.definition.commentary, 60));
            }
            println!();
        }
    }

    // show changed entries
    if !changed_entries.is_empty() {
        println!("{} changed commentaries:\n", changed_entries.len());
        for entry in &changed_entries {
            println!("  {}:{}", entry.definition.file, entry.definition.line);
            println!("    ^{}^", truncate(&entry.definition.mantra_text, 60));
            if let DefinitionStatus::Changed { old_commentary } = &entry.status {
                println!("    - was: \"{}\"", truncate(old_commentary, 50));
                println!(
                    "    + now: \"{}\"",
                    truncate(&entry.definition.commentary, 50)
                );
            }
            println!();
        }
    }

    println!(
        "summary: {} accepted, {} new, {} changed",
        accepted_count,
        new_entries.len(),
        changed_entries.len()
    );
    println!("\nuse 'vyasa add' to accept changes.");

    Ok(())
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
