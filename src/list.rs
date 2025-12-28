use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_canon, Canon, CanonSearchResult, MantraStatus};
use std::path::Path;

pub fn run(path: &Path, filter: Option<String>, pending_only: bool) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;

    let canon = match Canon::find(&repo_root) {
        CanonSearchResult::NotFound => Canon::default(),
        CanonSearchResult::Found(canon) => canon,
        CanonSearchResult::Multiple(paths) => {
            return Err(format!(
                "multiple canon.md files found:\n  {}\n\na kosha must have exactly one canon.md at the root",
                paths.join("\n  ")
            ));
        }
        CanonSearchResult::Invalid { path, errors } => {
            return Err(format!(
                "invalid canon.md at {}:\n  {}",
                path,
                errors.join("\n  ")
            ));
        }
    };

    let mantras = compare_with_canon(&repo, &canon);

    // apply filter if provided
    let filtered: Vec<_> = if let Some(ref f) = filter {
        mantras
            .iter()
            .filter(|m| m.mantra_text.contains(f))
            .collect()
    } else {
        mantras.iter().collect()
    };

    // calculate stats
    let mut accepted = 0;
    let mut new_count = 0;
    let mut changed = 0;
    let mut orphaned = 0;

    let display_list: Vec<_> = filtered
        .iter()
        .map(|m| {
            let status = match &m.status {
                MantraStatus::Accepted => {
                    accepted += 1;
                    ListStatus::Accepted
                }
                MantraStatus::New => {
                    new_count += 1;
                    ListStatus::New
                }
                MantraStatus::Changed { .. } => {
                    changed += 1;
                    ListStatus::Changed
                }
                MantraStatus::OrphanedInCanon { .. } => {
                    orphaned += 1;
                    ListStatus::Orphaned
                }
            };
            (&m.mantra_text, status, m.bhasyas.len())
        })
        .collect();

    // filter to pending only if requested
    let display_list: Vec<_> = if pending_only {
        display_list
            .into_iter()
            .filter(|(_, status, _)| !matches!(status, ListStatus::Accepted))
            .collect()
    } else {
        display_list
    };

    // sort alphabetically
    let mut sorted: Vec<_> = display_list;
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    // print summary
    if let Some(ref f) = filter {
        println!(
            "{} mantras matching \"{}\" ({} in canon, {} new, {} changed, {} orphaned)\n",
            sorted.len(),
            f,
            accepted,
            new_count,
            changed,
            orphaned
        );
    } else {
        println!(
            "{} mantras ({} in canon, {} new, {} changed, {} orphaned)\n",
            sorted.len(),
            accepted,
            new_count,
            changed,
            orphaned
        );
    }

    // print list
    for (mantra, status, count) in &sorted {
        let marker = match status {
            ListStatus::Accepted => "[ok]",
            ListStatus::New => "[!!]",
            ListStatus::Changed => "[**]",
            ListStatus::Orphaned => "[??]",
        };

        let suffix = if *count > 1 {
            format!(" ({} bhasyas)", count)
        } else if *count == 0 {
            " (canon only)".to_string()
        } else {
            String::new()
        };

        println!("  {} {}{}", marker, truncate(mantra, 55), suffix);
    }

    Ok(())
}

#[derive(Debug)]
enum ListStatus {
    Accepted,
    New,
    Changed,
    Orphaned,
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
