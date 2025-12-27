use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_canon, Canon, CanonSearchResult, MantraStatus};
use std::collections::HashMap;
use std::path::Path;

pub fn run(path: &Path, verbose: bool) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;

    let canon = match Canon::find(&repo_root) {
        CanonSearchResult::NotFound => {
            println!("no canon.md found\n");
            println!("create a canon.md file at the repository root to track accepted mantras.");
            println!("format: filename.md followed by > **^mantra^** - commentary\n");
            Canon::default()
        }
        CanonSearchResult::Found(canon) => {
            if let Some(ref path) = canon.path {
                println!("using {}", path);
            }
            canon
        }
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

    let new_mantras: Vec<_> = mantras
        .iter()
        .filter(|m| matches!(m.status, MantraStatus::New))
        .collect();

    let changed_mantras: Vec<_> = mantras
        .iter()
        .filter(|m| matches!(m.status, MantraStatus::Changed { .. }))
        .collect();

    let orphaned_mantras: Vec<_> = mantras
        .iter()
        .filter(|m| matches!(m.status, MantraStatus::OrphanedInCanon { .. }))
        .collect();

    let accepted_count = mantras
        .iter()
        .filter(|m| matches!(m.status, MantraStatus::Accepted))
        .count();

    // show orphaned mantras first (these are errors)
    if !orphaned_mantras.is_empty() {
        println!("\n{} orphaned (in canon but not in source):", orphaned_mantras.len());
        for mantra in &orphaned_mantras {
            println!("  ^{}^", truncate(&mantra.mantra_text, 60));
        }
    }

    if new_mantras.is_empty() && changed_mantras.is_empty() && orphaned_mantras.is_empty() {
        println!("\nall {} mantras are in canon", mantras.len());
        return Ok(());
    }

    // show new mantras grouped by file
    if !new_mantras.is_empty() {
        if verbose {
            println!("\n{} pending:\n", new_mantras.len());
            for mantra in &new_mantras {
                println!("  ^{}^", truncate(&mantra.mantra_text, 60));
                if let Some(def) = mantra.definitions.first() {
                    println!("    {}:{}", def.file, def.line);
                    if !def.commentary.is_empty() {
                        println!("    \"{}\"", truncate(&def.commentary, 60));
                    }
                }
                if mantra.definitions.len() > 1 {
                    println!("    ({} definitions total)", mantra.definitions.len());
                }
                println!();
            }
        } else {
            // compact view: group by file (count all definitions, not just first)
            let mut by_file: HashMap<String, usize> = HashMap::new();
            for mantra in &new_mantras {
                for def in &mantra.definitions {
                    *by_file.entry(def.file.clone()).or_insert(0) += 1;
                }
            }

            let mut files: Vec<_> = by_file.into_iter().collect();
            files.sort_by(|a, b| b.1.cmp(&a.1)); // sort by count descending

            println!("\n{} pending ({} definitions):", new_mantras.len(),
                files.iter().map(|(_, c)| c).sum::<usize>());
            for (file, count) in &files {
                println!("  {:>3}  {}", count, file);
            }
        }
    }

    // show changed mantras
    if !changed_mantras.is_empty() {
        if verbose {
            println!("\n{} changed:\n", changed_mantras.len());
            for mantra in &changed_mantras {
                println!("  ^{}^", truncate(&mantra.mantra_text, 60));
                if let MantraStatus::Changed { canon_commentary } = &mantra.status {
                    println!("    canon: \"{}\"", truncate(canon_commentary, 50));
                    if let Some(def) = mantra.definitions.first() {
                        println!("    now:   \"{}\"", truncate(&def.commentary, 50));
                    }
                }
                println!();
            }
        } else {
            // compact view: group by file (count all definitions, not just first)
            let mut by_file: HashMap<String, usize> = HashMap::new();
            for mantra in &changed_mantras {
                for def in &mantra.definitions {
                    *by_file.entry(def.file.clone()).or_insert(0) += 1;
                }
            }

            let mut files: Vec<_> = by_file.into_iter().collect();
            files.sort_by(|a, b| b.1.cmp(&a.1));

            println!("\n{} changed ({} definitions):", changed_mantras.len(),
                files.iter().map(|(_, c)| c).sum::<usize>());
            for (file, count) in &files {
                println!("  {:>3}  {}", count, file);
            }
        }
    }

    println!(
        "\nsummary: {} accepted, {} pending, {} changed, {} orphaned",
        accepted_count,
        new_mantras.len(),
        changed_mantras.len(),
        orphaned_mantras.len()
    );

    if !orphaned_mantras.is_empty() {
        return Err("canon contains mantras not found in source files".to_string());
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
