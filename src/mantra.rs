use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_canon, Canon, CanonSearchResult, MantraStatus};
use std::path::Path;

pub fn run(path: &Path, mantra_text: &str, show_references: bool) -> Result<(), String> {
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

    // find mantra matching this text
    let found = mantras.iter().find(|m| m.mantra_text == mantra_text);

    if found.is_none() {
        // try partial match
        let partial: Vec<_> = mantras
            .iter()
            .filter(|m| m.mantra_text.contains(mantra_text))
            .collect();

        if partial.is_empty() {
            return Err(format!("no mantra matches '{}'", mantra_text));
        }

        println!("no exact match, did you mean one of these?\n");
        for m in &partial {
            println!("  ^{}^", truncate(&m.mantra_text, 60));
            if let Some(def) = m.bhasyas.first() {
                println!("    {}:{}", def.file, def.line);
            }
        }
        return Ok(());
    }

    let mantra = found.unwrap();

    println!("mantra: {}\n", mantra.mantra_text);

    match &mantra.status {
        MantraStatus::Accepted => {
            println!("status: in canon\n");
            if let Some(canon_entry) = canon.get(&mantra.mantra_text) {
                println!("canon commentary:");
                println!("  \"{}\"\n", truncate(&canon_entry.commentary, 70));
            }
        }
        MantraStatus::New => {
            println!("status: not yet in canon\n");
        }
        MantraStatus::Changed { canon_commentary } => {
            println!("status: commentary changed from canon\n");
            println!("canon commentary:");
            println!("  \"{}\"\n", truncate(canon_commentary, 70));
        }
        MantraStatus::OrphanedInCanon { canon_commentary } => {
            println!("status: ERROR - only in canon, not in source files\n");
            println!("canon commentary:");
            println!("  \"{}\"\n", truncate(canon_commentary, 70));
            println!("this mantra must be defined in a source file, not just canon.md\n");
        }
    }

    if !mantra.bhasyas.is_empty() {
        println!("bhasyas ({}):", mantra.bhasyas.len());
        for def in &mantra.bhasyas {
            println!("  {}:{}", def.file, def.line);
            if !def.commentary.is_empty() {
                println!("    \"{}\"", truncate(&def.commentary, 60));
            }
            println!();
        }
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
