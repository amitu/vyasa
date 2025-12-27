use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_snapshot, DefinitionStatus, Snapshot};
use std::collections::HashMap;
use std::path::Path;

pub fn run(path: &Path, filter: Option<String>, pending_only: bool) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;
    let snapshot = Snapshot::load(&repo_root);

    let statuses = compare_with_snapshot(&repo, &snapshot);

    // group by mantra text
    let mut by_mantra: HashMap<String, Vec<_>> = HashMap::new();
    for status in &statuses {
        by_mantra
            .entry(status.definition.mantra_text.clone())
            .or_default()
            .push(status);
    }

    // apply filter if provided
    let filtered: Vec<_> = if let Some(ref f) = filter {
        by_mantra
            .iter()
            .filter(|(mantra, _)| mantra.contains(f))
            .collect()
    } else {
        by_mantra.iter().collect()
    };

    // calculate stats
    let mut fully_accepted = 0;
    let mut pending = 0;
    let mut partial = 0;

    let mut display_list: Vec<_> = filtered
        .iter()
        .map(|(mantra, entries)| {
            let accepted = entries
                .iter()
                .filter(|e| matches!(e.status, DefinitionStatus::Accepted))
                .count();
            let total = entries.len();
            let pending_count = total - accepted;

            let status = if pending_count == 0 {
                fully_accepted += 1;
                MantraStatus::Accepted
            } else if accepted == 0 {
                pending += 1;
                MantraStatus::Pending
            } else {
                partial += 1;
                MantraStatus::Partial { accepted, total }
            };

            (mantra.as_str(), status, entries.len())
        })
        .collect();

    // filter to pending only if requested
    if pending_only {
        display_list.retain(|(_, status, _)| !matches!(status, MantraStatus::Accepted));
    }

    // sort alphabetically
    display_list.sort_by(|a, b| a.0.cmp(b.0));

    // print summary
    if let Some(ref f) = filter {
        println!(
            "{} mantras matching \"{}\" ({} accepted, {} pending, {} partial)\n",
            display_list.len(),
            f,
            fully_accepted,
            pending,
            partial
        );
    } else {
        println!(
            "{} mantras ({} accepted, {} pending, {} partial)\n",
            display_list.len(),
            fully_accepted,
            pending,
            partial
        );
    }

    // print list
    for (mantra, status, count) in &display_list {
        let marker = match status {
            MantraStatus::Accepted => "[ok]",
            MantraStatus::Pending => "[!!]",
            MantraStatus::Partial { .. } => "[**]",
        };

        let suffix = match status {
            MantraStatus::Accepted if *count > 1 => format!(" ({} commentaries)", count),
            MantraStatus::Partial { accepted, total } => {
                format!(" ({}/{} accepted)", accepted, total)
            }
            MantraStatus::Pending if *count > 1 => format!(" ({} pending)", count),
            _ => String::new(),
        };

        println!("  {} {}{}", marker, truncate(mantra, 55), suffix);
    }

    Ok(())
}

#[derive(Debug)]
enum MantraStatus {
    Accepted,
    Pending,
    Partial { accepted: usize, total: usize },
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
