use crate::parser::Repository;
use std::path::Path;

pub fn run(path: &Path, mantra_text: &str, show_references: bool) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    // find mantra matching this text
    let found = repo.mantras.get(mantra_text);

    if found.is_none() {
        // try partial match
        let partial: Vec<_> = repo
            .mantras
            .keys()
            .filter(|m| m.contains(mantra_text))
            .collect();

        if partial.is_empty() {
            return Err(format!("no mantra matches '{}'", mantra_text));
        }

        println!("no exact match, did you mean one of these?\n");
        for m in &partial {
            println!("  ^{}^", truncate(m, 60));
            if let Some(mantra) = repo.mantras.get(*m) {
                println!("    {}:{}", mantra.file, mantra.line);
            }
        }
        return Ok(());
    }

    let mantra = found.unwrap();

    println!("mantra: {}\n", mantra_text);

    // show bhasyas
    let bhasyas: Vec<_> = repo
        .bhasyas
        .iter()
        .filter(|b| b.mantra_text == mantra_text)
        .collect();

    if bhasyas.is_empty() {
        println!("bhasyas: none (no explanations)\n");
    } else {
        println!("bhasyas ({}):", bhasyas.len());
        for b in &bhasyas {
            println!("  {}:{}", b.file, b.line);
            if !b.commentary.is_empty() {
                println!("    \"{}\"", truncate(&b.commentary, 60));
            }
            println!();
        }
    }

    // show the mula definition location
    println!("mula definition: {}:{}\n", mantra.file, mantra.line);

    if show_references {
        // find all anusrits to this mantra
        let refs: Vec<_> = repo
            .anusrits
            .iter()
            .filter(|r| r.mantra_text == mantra_text)
            .collect();

        if refs.is_empty() {
            println!("anusrits: none");
        } else {
            println!("anusrits ({}):", refs.len());
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
