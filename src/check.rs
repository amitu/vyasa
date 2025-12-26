use crate::parser::Repository;
use std::collections::HashSet;
use std::path::Path;

// ~vyasa check exits with non zero exit code if any rule is violated~
pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let unexplained = repo.unexplained_mantras();

    let mut has_errors = false;
    let mut error_counts = Vec::new();

    // check for unexplained mantras
    if !unexplained.is_empty() {
        has_errors = true;
        println!(
            "found {} mantras without explanations:\n",
            unexplained.len()
        );
        for mantra in &unexplained {
            println!("  {}:{}", mantra.file, mantra.line);
            println!("    ^{}^\n", truncate(&mantra.text, 60));
        }
        error_counts.push(format!("{} unexplained mantras", unexplained.len()));
    }

    // ~vyasa check reports undefined references~
    let undefined_refs = check_undefined_references(&repo);
    if !undefined_refs.is_empty() {
        has_errors = true;
        println!(
            "found {} undefined references:\n",
            undefined_refs.len()
        );
        for (file, line, text) in &undefined_refs {
            println!("  {}:{}", file, line);
            println!("    ~{}~\n", truncate(text, 60));
        }
        error_counts.push(format!("{} undefined references", undefined_refs.len()));
    }

    // ~kosha check verifies all kosha references~
    let kosha_errors = check_kosha_references(&repo);
    if !kosha_errors.is_empty() {
        has_errors = true;
        println!("found {} kosha reference errors:\n", kosha_errors.len());
        for error in &kosha_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} kosha errors", kosha_errors.len()));
    }

    if has_errors {
        Err(error_counts.join(", "))
    } else {
        println!("all {} mantras validated successfully", repo.mantras.len());
        Ok(())
    }
}

// ~kosha check verifies all kosha references~
fn check_kosha_references(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // collect all defined kosha aliases
    let defined_aliases: HashSet<_> = repo
        .kosha_config
        .aliases
        .iter()
        .map(|a| a.alias.as_str())
        .collect();

    // collect all local dir mappings
    let local_dirs: HashSet<_> = repo
        .kosha_config
        .local_dirs
        .iter()
        .map(|d| d.alias.as_str())
        .collect();

    // check each reference with a kosha
    for reference in &repo.references {
        if let Some(kosha_name) = &reference.kosha {
            // check if alias is defined
            if !defined_aliases.contains(kosha_name.as_str()) {
                errors.push(format!(
                    "{}:{}: undefined kosha '{}' in ~{}~@{}",
                    reference.file,
                    reference.line,
                    kosha_name,
                    truncate(&reference.mantra_text, 30),
                    kosha_name
                ));
                continue;
            }

            // find the kosha value
            if let Some(alias) = repo.kosha_config.aliases.iter().find(|a| &a.alias == kosha_name) {
                let is_folder = alias.value.starts_with('/')
                    || alias.value.starts_with("./")
                    || alias.value.starts_with("../");

                if !is_folder && !local_dirs.contains(kosha_name.as_str()) {
                    errors.push(format!(
                        "kosha '{}' refers to '{}' but no local folder defined in kosha.local.vyasa",
                        kosha_name, alias.value
                    ));
                }

                // check if resolved path exists
                if let Some(resolved_path) = repo.resolve_kosha_path(kosha_name) {
                    let path = Path::new(&resolved_path);
                    if (resolved_path.starts_with('/')
                        || resolved_path.starts_with("./")
                        || resolved_path.starts_with("../"))
                        && !path.exists()
                    {
                        errors.push(format!(
                            "kosha '{}' folder does not exist: {}",
                            kosha_name, resolved_path
                        ));
                    }
                }
            }
        }
    }

    errors
}

// ~vyasa check reports undefined references~
fn check_undefined_references(repo: &Repository) -> Vec<(String, usize, String)> {
    let mut undefined = Vec::new();

    for reference in &repo.references {
        // skip external kosha references (checked separately)
        if reference.kosha.is_some() {
            continue;
        }

        // check if reference matches a mantra (exact or template)
        let is_defined = repo.mantras.contains_key(&reference.mantra_text)
            || reference.matched_template.is_some();

        if !is_defined {
            undefined.push((
                reference.file.clone(),
                reference.line,
                reference.mantra_text.clone(),
            ));
        }
    }

    undefined
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
