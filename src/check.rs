use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{Canon, CanonSearchResult};
use std::path::Path;

// _| vyasa check exits with non zero exit code if any rule is violated |_
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

    // _| vyasa check reports undefined anusrits |_
    let undefined_refs = check_undefined_anusrits(&repo);
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

    // _| kosha check verifies all kosha anusrits |_
    let kosha_errors = check_kosha_anusrits(&repo, path);
    if !kosha_errors.is_empty() {
        has_errors = true;
        println!("found {} kosha anusrit errors:\n", kosha_errors.len());
        for error in &kosha_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} kosha errors", kosha_errors.len()));
    }

    // _| check validates canon entries exist in source files |_
    let canon_errors = check_canon(&repo, path);
    if !canon_errors.is_empty() {
        has_errors = true;
        println!("found {} canon errors:\n", canon_errors.len());
        for error in &canon_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} canon errors", canon_errors.len()));
    }

    if has_errors {
        Err(error_counts.join(", "))
    } else {
        println!("all {} mantras validated successfully", repo.mantras.len());
        Ok(())
    }
}

// _| kosha check verifies all kosha anusrits |_
// _| when a mantra from other kosha is referred, that mantra must exist in canon of that kosha |_
fn check_kosha_anusrits(repo: &Repository, repo_path: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    // load our own canon to check external entries
    let repo_root = find_repo_root(repo_path);
    let our_canon = repo_root
        .as_ref()
        .map(|r| Canon::find(r))
        .and_then(|result| match result {
            CanonSearchResult::Found(c) => Some(c),
            _ => None,
        });

    // cache loaded kosha canons
    let mut kosha_canons: std::collections::HashMap<String, Option<Canon>> =
        std::collections::HashMap::new();

    // helper to load a kosha's canon
    let load_kosha_canon = |kosha_name: &str, repo: &Repository| -> Option<Canon> {
        let resolved = repo.resolve_kosha_path(kosha_name)?;
        let kosha_path = Path::new(&resolved);
        if !kosha_path.exists() {
            return None;
        }
        match Canon::find(kosha_path) {
            CanonSearchResult::Found(c) => Some(c),
            _ => None,
        }
    };

    // check each anusrit with a kosha
    for anusrit in &repo.anusrits {
        if let Some(kosha_name) = &anusrit.kosha {
            // check if alias is defined in kosha.json
            let Some(kosha_path) = repo.kosha_config.aliases.get(kosha_name) else {
                errors.push(format!(
                    "{}:{}: undefined kosha '{}' in _{}_`@{}`",
                    anusrit.file,
                    anusrit.line,
                    kosha_name,
                    truncate(&anusrit.mantra_text, 30),
                    kosha_name
                ));
                continue;
            };

            // check if it's a local folder path
            let is_folder = kosha_path.starts_with('/')
                || kosha_path.starts_with("./")
                || kosha_path.starts_with("../");

            if !is_folder {
                errors.push(format!(
                    "kosha '{}' refers to '{}' - only local folder paths are currently supported",
                    kosha_name, kosha_path
                ));
                continue;
            }

            // check if resolved path exists
            let path = Path::new(kosha_path);
            if !path.exists() {
                errors.push(format!(
                    "kosha '{}' folder does not exist: {}",
                    kosha_name, kosha_path
                ));
                continue;
            }

            // load the kosha's canon and verify mantra exists
            let canon = kosha_canons
                .entry(kosha_name.clone())
                .or_insert_with(|| load_kosha_canon(kosha_name, repo));

            if let Some(canon) = canon {
                if canon.get(&anusrit.mantra_text).is_none() {
                    errors.push(format!(
                        "{}:{}: mantra not in {}'s canon: _{}_`@{}`",
                        anusrit.file,
                        anusrit.line,
                        kosha_name,
                        truncate(&anusrit.mantra_text, 30),
                        kosha_name
                    ));
                }
            } else {
                errors.push(format!(
                    "kosha '{}' has no canon.md",
                    kosha_name
                ));
            }
        }
    }

    // check external entries in our canon
    if let Some(canon) = our_canon {
        for entry in canon.external_entries() {
            if let Some(kosha_name) = &entry.external_kosha {
                // check if alias is defined in kosha.json
                if !repo.kosha_config.aliases.contains_key(kosha_name) {
                    errors.push(format!(
                        "canon anusrit references undefined kosha '{}' for ^{}^@{}",
                        kosha_name,
                        truncate(&entry.mantra, 30),
                        kosha_name
                    ));
                    continue;
                }

                // load the kosha's canon and verify mantra exists
                let external_canon = kosha_canons
                    .entry(kosha_name.clone())
                    .or_insert_with(|| load_kosha_canon(kosha_name, repo));

                if let Some(external_canon) = external_canon {
                    if external_canon.get(&entry.mantra).is_none() {
                        errors.push(format!(
                            "canon entry ^{}^@{} not found in {}'s canon",
                            truncate(&entry.mantra, 30),
                            kosha_name,
                            kosha_name
                        ));
                    }
                }
            }
        }
    }

    errors
}

// _| vyasa check reports undefined anusrits |_
fn check_undefined_anusrits(repo: &Repository) -> Vec<(String, usize, String)> {
    let mut undefined = Vec::new();

    for anusrit in &repo.anusrits {
        // skip external kosha anusrits (checked separately)
        if anusrit.kosha.is_some() {
            continue;
        }

        // check if anusrit matches a defined mantra
        if !repo.mantras.contains_key(&anusrit.mantra_text) {
            undefined.push((
                anusrit.file.clone(),
                anusrit.line,
                anusrit.mantra_text.clone(),
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

// _| check validates canon entries exist in source files |_
fn check_canon(repo: &Repository, repo_path: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    let repo_root = match find_repo_root(repo_path) {
        Some(r) => r,
        None => return errors, // no repo root, nothing to check
    };

    let canon = match Canon::find(&repo_root) {
        CanonSearchResult::Found(c) => c,
        CanonSearchResult::NotFound => return errors, // no canon, nothing to check
        CanonSearchResult::Multiple(paths) => {
            errors.push(format!(
                "multiple canon files found:\n    {}",
                paths.join("\n    ")
            ));
            return errors;
        }
        CanonSearchResult::Invalid { path, errors: errs } => {
            errors.push(format!(
                "invalid canon at {}:\n    {}",
                path,
                errs.join("\n    ")
            ));
            return errors;
        }
    };

    // check each canon entry exists in source files (not orphaned)
    for (mantra_text, entry) in &canon.entries {
        // skip external entries (those have @kosha suffix)
        if entry.external_kosha.is_some() {
            continue;
        }

        // check if this mantra exists in source definitions
        if !repo.mantras.contains_key(mantra_text) {
            errors.push(format!(
                "canon entry not found in source files: ^{}^",
                truncate(mantra_text, 50)
            ));
        }
    }

    errors
}
