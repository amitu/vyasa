use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_snapshot, DefinitionStatus, Snapshot};
use std::path::Path;

pub fn run(path: &Path, mantra_text: &str, show_references: bool) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;
    let snapshot = Snapshot::load(&repo_root);

    let statuses = compare_with_snapshot(&repo, &snapshot);

    // find all definitions matching this mantra
    let matches: Vec<_> = statuses
        .iter()
        .filter(|s| s.definition.mantra_text == mantra_text)
        .collect();

    if matches.is_empty() {
        // try partial match
        let partial: Vec<_> = statuses
            .iter()
            .filter(|s| s.definition.mantra_text.contains(mantra_text))
            .collect();

        if partial.is_empty() {
            return Err(format!("no mantra matches '{}'", mantra_text));
        }

        println!("no exact match, did you mean one of these?\n");
        for m in &partial {
            println!("  ^{}^", truncate(&m.definition.mantra_text, 60));
            println!("    {}:{}", m.definition.file, m.definition.line);
        }
        return Ok(());
    }

    // count accepted/pending
    let accepted_count = matches
        .iter()
        .filter(|s| matches!(s.status, DefinitionStatus::Accepted))
        .count();
    let pending_count = matches.len() - accepted_count;

    println!("mantra: {}\n", mantra_text);

    if pending_count == 0 {
        println!("status: fully accepted ({} commentaries)\n", matches.len());
    } else if accepted_count == 0 {
        println!("status: pending ({} commentaries)\n", matches.len());
    } else {
        println!(
            "status: partially accepted ({} of {} commentaries)\n",
            accepted_count,
            matches.len()
        );
    }

    println!("commentaries:");
    for entry in &matches {
        let status_marker = match &entry.status {
            DefinitionStatus::Accepted => "[accepted]",
            DefinitionStatus::New => "[pending]",
            DefinitionStatus::Changed { .. } => "[changed]",
        };

        println!(
            "  {} {}:{}",
            status_marker, entry.definition.file, entry.definition.line
        );
        if !entry.definition.commentary.is_empty() {
            println!("    \"{}\"", truncate(&entry.definition.commentary, 60));
        }

        if let DefinitionStatus::Changed { old_commentary } = &entry.status {
            println!("    was: \"{}\"", truncate(old_commentary, 60));
        }
        println!();
    }

    if show_references {
        // find all references to this mantra
        let refs: Vec<_> = repo
            .references
            .iter()
            .filter(|r| {
                r.mantra_text == mantra_text
                    || r.matched_template.as_ref() == Some(&mantra_text.to_string())
            })
            .collect();

        if refs.is_empty() {
            println!("references: none");
        } else {
            println!("references ({}):", refs.len());
            for r in &refs {
                println!("  {}:{}", r.file, r.line);
            }
        }
    }

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
