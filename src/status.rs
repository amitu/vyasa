use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_canon, Canon, CanonSearchResult, MantraStatus};
use std::path::Path;

pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;

    let canon = match Canon::find(&repo_root) {
        CanonSearchResult::NotFound => {
            println!("no canon.md found\n");
            println!("create a canon.md file at the repository root to track accepted mantras.");
            println!("format: ^mantra^ - commentary\n");
            Canon::default()
        }
        CanonSearchResult::Found(canon) => {
            if let Some(ref path) = canon.path {
                println!("using {}\n", path);
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
        println!("{} mantras in canon but not in any source file:\n", orphaned_mantras.len());
        for mantra in &orphaned_mantras {
            println!("  ^{}^", truncate(&mantra.mantra_text, 60));
            if let MantraStatus::OrphanedInCanon { canon_commentary } = &mantra.status {
                println!("    \"{}\"", truncate(canon_commentary, 60));
            }
            println!();
        }
        println!("canon is a digest only - mantras must be defined in source files.\n");
    }

    if new_mantras.is_empty() && changed_mantras.is_empty() && orphaned_mantras.is_empty() {
        println!("all {} mantras are in canon", mantras.len());
        return Ok(());
    }

    // show new mantras
    if !new_mantras.is_empty() {
        println!("{} new mantras not yet in canon:\n", new_mantras.len());
        for mantra in &new_mantras {
            println!("  ^{}^", truncate(&mantra.mantra_text, 60));
            // show first definition's location and commentary
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
    }

    // show changed mantras
    if !changed_mantras.is_empty() {
        println!("{} mantras with changed commentary:\n", changed_mantras.len());
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
    }

    println!(
        "summary: {} in canon, {} new, {} changed, {} orphaned",
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
