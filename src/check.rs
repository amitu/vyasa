use crate::parser::Repository;
use std::collections::HashMap;
use std::path::Path;

// _| vyasa check exits with non zero exit code if any rule is violated |_
pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    // require shastra name in .vyasa/config.json
    if repo.config.name.is_none() {
        return Err("missing 'name' in .vyasa/config.json - every shastra must have a name".to_string());
    }

    let unexplained = repo.unexplained_mantras();

    let mut has_errors = false;
    let mut has_warnings = false;
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
    let (shastra_errors, shastra_warnings) = check_shastra_quotes(&repo);
    if !shastra_errors.is_empty() {
        has_errors = true;
        println!("found {} shastra quote errors:\n", shastra_errors.len());
        for error in &shastra_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} shastra quote errors", shastra_errors.len()));
    }
    if !shastra_warnings.is_empty() {
        has_warnings = true;
        println!("found {} shastra quote warnings:\n", shastra_warnings.len());
        for warning in &shastra_warnings {
            println!("  {}\n", warning);
        }
    }

    if has_errors {
        Err(error_counts.join(", "))
    } else {
        if has_warnings {
            println!("all {} mantras validated with warnings", repo.mantras.len());
        } else {
            println!("all {} mantras validated successfully", repo.mantras.len());
        }
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

/// Check shastra-quoted bhasyas: verify they exist in source and warn if tyakta
fn check_shastra_quotes(repo: &Repository) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    // find all bhasyas with shastra attribution
    for bhasya in &repo.bhasyas {
        if let Some(shastra_name) = &bhasya.shastra {
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
                    // only tyakta bhasya exists - warning
                    warnings.push(format!(
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

    (errors, warnings)
}

// _| vyasa check reports undefined anusrits |_
fn check_undefined_anusrits(repo: &Repository) -> Vec<(String, usize, String)> {
    let mut undefined = Vec::new();

    for anusrit in &repo.anusrits {
        // skip external shastra anusrits (checked separately)
        if anusrit.shastra.is_some() {
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
