use crate::parser::Repository;
use std::collections::HashMap;
use std::path::Path;

// _| vyasa exits with non zero exit code if any rule is violated |_
pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    // require shastra name in .vyasa/config.json
    if repo.config.name.is_none() {
        return Err("missing 'name' in .vyasa/config.json - every shastra must have a name".to_string());
    }

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

    // check for duplicate bhasyas (same mantra + commentary)
    let shastra_name = repo.config.name.as_ref().unwrap();
    let duplicate_bhasyas = check_duplicate_bhasyas(&repo);
    if !duplicate_bhasyas.is_empty() {
        has_errors = true;
        println!(
            "found {} duplicate bhasyas:\n",
            duplicate_bhasyas.len()
        );
        for (mantra, commentary, locations) in &duplicate_bhasyas {
            println!("  > **^{}^**", mantra);
            println!("  > {}\n", truncate(commentary, 70));
            println!("  found at:");
            for (file, line) in locations {
                println!("    - {}:{}", file, line);
            }
            println!("\n  use `shastra: {}` to quote from canonical location\n", shastra_name);
        }
        error_counts.push(format!("{} duplicate bhasyas", duplicate_bhasyas.len()));
    }

    // _| vyasa reports undefined anusrits |_
    let (undefined_refs, ambiguous_refs) = check_undefined_anusrits(&repo);
    if !undefined_refs.is_empty() {
        has_errors = true;
        println!(
            "found {} undefined anusrits:\n",
            undefined_refs.len()
        );
        for (file, line, text) in &undefined_refs {
            println!("  {}:{}", file, line);
            println!("    anusrit: {}\n", truncate(text, 60));
        }
        error_counts.push(format!("{} undefined anusrits", undefined_refs.len()));
    }

    if !ambiguous_refs.is_empty() {
        has_errors = true;
        println!(
            "found {} ambiguous anusrits:\n",
            ambiguous_refs.len()
        );
        for (file, line, text, shastras) in &ambiguous_refs {
            println!("  {}:{}", file, line);
            println!("    ^{}^", truncate(text, 60));
            println!("    found in: {}", shastras.join(", "));
            println!("    use @shastra to disambiguate\n");
        }
        error_counts.push(format!("{} ambiguous anusrits", ambiguous_refs.len()));
    }

    // check external shastra anusrits
    let shastra_anusrit_errors = check_shastra_anusrits(&repo);
    if !shastra_anusrit_errors.is_empty() {
        has_errors = true;
        println!("found {} shastra anusrit errors:\n", shastra_anusrit_errors.len());
        for error in &shastra_anusrit_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} shastra anusrit errors", shastra_anusrit_errors.len()));
    }

    // check shastra-quoted bhasyas
    let shastra_errors = check_shastra_quotes(&repo);
    if !shastra_errors.is_empty() {
        has_errors = true;
        println!("found {} shastra quote errors:\n", shastra_errors.len());
        for error in &shastra_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} shastra quote errors", shastra_errors.len()));
    }

    // check khandita (refuted) bhasyas
    let (khandita_errors, khandita_notes) = check_khandita(&repo);
    if !khandita_errors.is_empty() {
        has_errors = true;
        println!("found {} khandita errors:\n", khandita_errors.len());
        for error in &khandita_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} khandita errors", khandita_errors.len()));
    }
    if !khandita_notes.is_empty() {
        println!("found {} khandita notes:\n", khandita_notes.len());
        for note in &khandita_notes {
            println!("  {}\n", note);
        }
    }

    // check for conflicting khandita/uddhrit (can't both refute and quote same bhasya)
    let conflict_errors = check_khandita_uddhrit_conflicts(&repo);
    if !conflict_errors.is_empty() {
        has_errors = true;
        println!("found {} khandita/uddhrit conflicts:\n", conflict_errors.len());
        for error in &conflict_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} khandita/uddhrit conflicts", conflict_errors.len()));
    }

    if has_errors {
        Err(error_counts.join(", "))
    } else {
        Ok(())
    }
}

/// Check external shastra anusrits: verify alias, path, and mantra exists in mula form
fn check_shastra_anusrits(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    // check each anusrit with a shastra reference
    for anusrit in &repo.anusrits {
        if let Some(shastra_name) = &anusrit.shastra {
            // check if alias is defined in shastra.json
            let Some(shastra_path) = repo.shastra_config.aliases.get(shastra_name) else {
                errors.push(format!(
                    "{}:{}: undefined shastra '{}' in anusrit @{}",
                    anusrit.file,
                    anusrit.line,
                    shastra_name,
                    shastra_name
                ));
                continue;
            };

            // check if it's a local folder path
            let is_folder = shastra_path.starts_with('/')
                || shastra_path.starts_with("./")
                || shastra_path.starts_with("../");

            if !is_folder {
                errors.push(format!(
                    "shastra '{}' refers to '{}' - only local folder paths are currently supported",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // check if resolved path exists
            let path = Path::new(shastra_path);
            if !path.exists() {
                errors.push(format!(
                    "shastra '{}' folder does not exist: {}",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // load the external shastra repo
            let external_repo = shastra_repos
                .entry(shastra_name.clone())
                .or_insert_with(|| Repository::parse(path).ok());

            if let Some(external) = external_repo {
                // check if the mantra exists in mula form (not tyakta-only)
                let mantra_exists = external.mantras.contains_key(&anusrit.mantra_text);

                if !mantra_exists {
                    errors.push(format!(
                        "{}:{}: mantra not found in shastra '{}': ^{}^",
                        anusrit.file,
                        anusrit.line,
                        shastra_name,
                        truncate(&anusrit.mantra_text, 30)
                    ));
                }
            } else {
                errors.push(format!(
                    "failed to parse shastra '{}' at {}",
                    shastra_name, shastra_path
                ));
            }
        }
    }

    errors
}

/// Check shastra-quoted bhasyas: verify they exist in source, error if tyakta
fn check_shastra_quotes(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    let self_name = repo.config.name.as_ref().map(|s| s.as_str());

    // find all bhasyas with shastra attribution
    for bhasya in &repo.bhasyas {
        if let Some(shastra_name) = &bhasya.shastra {
            // check if this is a self-reference
            if self_name == Some(shastra_name.as_str()) {
                // self-reference: check against current repo
                let has_mula = repo.mantras.contains_key(&bhasya.mantra_text);
                if !has_mula {
                    // check if it exists as tyakta only
                    let has_any = repo.bhasyas.iter().any(|b| {
                        b.mantra_text == bhasya.mantra_text && b.shastra.is_none()
                    });
                    if !has_any {
                        errors.push(format!(
                            "{}:{}: mantra not found in self: ^{}^",
                            bhasya.file,
                            bhasya.line,
                            truncate(&bhasya.mantra_text, 30)
                        ));
                    }
                }
                continue;
            }

            // resolve shastra name to path via shastra.json
            let Some(shastra_path) = repo.shastra_config.aliases.get(shastra_name) else {
                errors.push(format!(
                    "{}:{}: undefined shastra '{}' for quoted ^{}^",
                    bhasya.file,
                    bhasya.line,
                    shastra_name,
                    truncate(&bhasya.mantra_text, 30)
                ));
                continue;
            };

            // check if it's a local folder path
            let is_folder = shastra_path.starts_with('/')
                || shastra_path.starts_with("./")
                || shastra_path.starts_with("../");

            if !is_folder {
                errors.push(format!(
                    "shastra '{}' refers to '{}' - only local folder paths are currently supported",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // check if resolved path exists
            let path = Path::new(shastra_path);
            if !path.exists() {
                errors.push(format!(
                    "shastra '{}' folder does not exist: {}",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // load the external shastra repo
            let external_repo = shastra_repos
                .entry(shastra_name.clone())
                .or_insert_with(|| Repository::parse(path).ok());

            if let Some(external) = external_repo {
                // check if mantra exists in mula form (non-tyakta bhasya)
                let has_mula = external.mantras.contains_key(&bhasya.mantra_text);
                // check if any bhasya (mula or tyakta) exists
                let has_any_bhasya = external.bhasyas.iter().any(|b| {
                    b.mantra_text == bhasya.mantra_text
                });

                if !has_any_bhasya {
                    // no bhasya at all - error
                    errors.push(format!(
                        "{}:{}: mantra not found in shastra '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(&bhasya.mantra_text, 30)
                    ));
                    continue;
                }

                if !has_mula {
                    // only tyakta bhasya exists - error
                    errors.push(format!(
                        "{}:{}: quoted tyakta from '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(&bhasya.mantra_text, 30)
                    ));
                }
            } else {
                errors.push(format!(
                    "failed to parse shastra '{}' at {}",
                    shastra_name, shastra_path
                ));
            }
        }
    }

    errors
}

/// Check khandita (refuted) bhasyas: verify they exist in source shastra
/// Returns: (errors, notes) - notes inform when source has tyakta'd the bhasya
fn check_khandita(repo: &Repository) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut notes = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    // find all bhasyas with khandita attribution
    for bhasya in &repo.bhasyas {
        if let Some(shastra_name) = &bhasya.khandita {
            // resolve shastra name to path via shastra.json
            let Some(shastra_path) = repo.shastra_config.aliases.get(shastra_name) else {
                errors.push(format!(
                    "{}:{}: undefined shastra '{}' for khandita ^{}^",
                    bhasya.file,
                    bhasya.line,
                    shastra_name,
                    truncate(&bhasya.mantra_text, 30)
                ));
                continue;
            };

            // check if it's a local folder path
            let is_folder = shastra_path.starts_with('/')
                || shastra_path.starts_with("./")
                || shastra_path.starts_with("../");

            if !is_folder {
                errors.push(format!(
                    "shastra '{}' refers to '{}' - only local folder paths are currently supported",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // check if resolved path exists
            let path = Path::new(shastra_path);
            if !path.exists() {
                errors.push(format!(
                    "shastra '{}' folder does not exist: {}",
                    shastra_name, shastra_path
                ));
                continue;
            }

            // load the external shastra repo
            let external_repo = shastra_repos
                .entry(shastra_name.clone())
                .or_insert_with(|| Repository::parse(path).ok());

            if let Some(external) = external_repo {
                // check if any bhasya exists for this mantra
                let has_any_bhasya = external.bhasyas.iter().any(|b| {
                    b.mantra_text == bhasya.mantra_text
                });

                if !has_any_bhasya {
                    // no bhasya at all - error: can't refute what doesn't exist
                    errors.push(format!(
                        "{}:{}: cannot khandita non-existent bhasya from '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(&bhasya.mantra_text, 30)
                    ));
                    continue;
                }

                // check if it's mula (non-tyakta) in source
                let has_mula = external.mantras.contains_key(&bhasya.mantra_text);
                if !has_mula {
                    // only tyakta bhasya exists - note: they already abandoned it
                    notes.push(format!(
                        "{}:{}: khandita bhasya is already tyakta in '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(&bhasya.mantra_text, 30)
                    ));
                }
            } else {
                errors.push(format!(
                    "failed to parse shastra '{}' at {}",
                    shastra_name, shastra_path
                ));
            }
        }
    }

    (errors, notes)
}

/// Check that same bhasya is not both khandita and uddhrit from same shastra
/// If you refute a bhasya, you must refute it consistently - no quoting it elsewhere
fn check_khandita_uddhrit_conflicts(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // collect all khandita: (mantra_text, shastra) -> (file, line)
    let mut khandita_refs: HashMap<(&str, &str), (&str, usize)> = HashMap::new();
    for bhasya in &repo.bhasyas {
        if let Some(ref shastra) = bhasya.khandita {
            khandita_refs.insert(
                (&bhasya.mantra_text, shastra),
                (&bhasya.file, bhasya.line),
            );
        }
    }

    // check if any uddhrit matches a khandita
    for bhasya in &repo.bhasyas {
        if let Some(ref shastra) = bhasya.shastra {
            let key = (bhasya.mantra_text.as_str(), shastra.as_str());
            if let Some((khandita_file, khandita_line)) = khandita_refs.get(&key) {
                errors.push(format!(
                    "{}:{}: cannot uddhrit ^{}^ from '{}' - already khandita at {}:{}",
                    bhasya.file,
                    bhasya.line,
                    truncate(&bhasya.mantra_text, 30),
                    shastra,
                    khandita_file,
                    khandita_line
                ));
            }
        }
    }

    errors
}

/// Check for duplicate bhasyas - same mantra + commentary must be unique
/// Returns: Vec<(mantra_text, commentary, locations)>
fn check_duplicate_bhasyas(repo: &Repository) -> Vec<(String, String, Vec<(String, usize)>)> {
    // key: (mantra_text, commentary) -> list of (file, line)
    let mut occurrences: HashMap<(&str, &str), Vec<(&str, usize)>> = HashMap::new();

    for bhasya in &repo.bhasyas {
        // skip uddhrit (quoted from other shastras) - duplicates allowed
        if bhasya.shastra.is_some() {
            continue;
        }

        let key = (bhasya.mantra_text.as_str(), bhasya.commentary.as_str());
        occurrences
            .entry(key)
            .or_default()
            .push((&bhasya.file, bhasya.line));
    }

    // collect only those with more than one occurrence
    occurrences
        .into_iter()
        .filter(|(_, locs)| locs.len() > 1)
        .map(|((mantra, commentary), locs)| {
            (
                mantra.to_string(),
                commentary.to_string(),
                locs.into_iter().map(|(f, l)| (f.to_string(), l)).collect(),
            )
        })
        .collect()
}

// _| vyasa reports undefined anusrits |_
fn check_undefined_anusrits(repo: &Repository) -> (Vec<(String, usize, String)>, Vec<(String, usize, String, Vec<String>)>) {
    let mut undefined = Vec::new();
    let mut ambiguous = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    for anusrit in &repo.anusrits {
        // explicit @shastra anusrits are checked separately
        if anusrit.shastra.is_some() {
            continue;
        }

        // first check current shastra
        if repo.mantras.contains_key(&anusrit.mantra_text) {
            continue;
        }

        // not in current shastra - check all external shastras
        let mut found_in: Vec<String> = Vec::new();

        for (shastra_name, shastra_path) in &repo.shastra_config.aliases {
            // only check local folder paths
            let is_folder = shastra_path.starts_with('/')
                || shastra_path.starts_with("./")
                || shastra_path.starts_with("../");

            if !is_folder {
                continue;
            }

            let path = Path::new(shastra_path);
            if !path.exists() {
                continue;
            }

            // load external shastra
            let external_repo = shastra_repos
                .entry(shastra_name.clone())
                .or_insert_with(|| Repository::parse(path).ok());

            if let Some(external) = external_repo {
                if external.mantras.contains_key(&anusrit.mantra_text) {
                    found_in.push(shastra_name.clone());
                }
            }
        }

        if found_in.is_empty() {
            // not found anywhere
            undefined.push((
                anusrit.file.clone(),
                anusrit.line,
                anusrit.mantra_text.clone(),
            ));
        } else if found_in.len() > 1 {
            // found in multiple shastras - ambiguous
            ambiguous.push((
                anusrit.file.clone(),
                anusrit.line,
                anusrit.mantra_text.clone(),
                found_in,
            ));
        }
        // found in exactly one external shastra - valid, no error
    }

    (undefined, ambiguous)
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
