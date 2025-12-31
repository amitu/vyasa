use crate::parser::{Repository, BhasyaKind};
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
        for (mantra_text, info) in &unexplained {
            println!("  {}:{}", info.file, info.line);
            println!("    ^{}^\n", truncate(mantra_text, 60));
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
        for (_mantra, paragraph, locations) in &duplicate_bhasyas {
            // show the original paragraph with indentation
            for line in paragraph.lines().take(4) {
                println!("  {}", line);
            }
            if paragraph.lines().count() > 4 {
                println!("  ...");
            }
            println!();
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
    let khandita_errors = check_khandita(&repo);
    if !khandita_errors.is_empty() {
        has_errors = true;
        println!("found {} khandita errors:\n", khandita_errors.len());
        for error in &khandita_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} khandita errors", khandita_errors.len()));
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

    // check for unresolved conflicts between shastras I follow
    let unresolved_errors = check_unresolved_shastra_conflicts(&repo);
    if !unresolved_errors.is_empty() {
        has_errors = true;
        println!("found {} unresolved shastra conflicts:\n", unresolved_errors.len());
        for error in &unresolved_errors {
            println!("  {}\n", error);
        }
        error_counts.push(format!("{} unresolved shastra conflicts", unresolved_errors.len()));
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

    // find all mula mantras in bhasyas with Uddhrit kind
    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        if let BhasyaKind::Uddhrit(ref shastra_name) = bhasya.kind {
            // check if this is a self-reference
            if self_name == Some(shastra_name.as_str()) {
                // self-reference: check against current repo
                let has_mula = repo.mantras.contains_key(mantra_text);
                if !has_mula {
                    // check if it exists as tyakta only
                    let has_any = repo.has_any_bhasya_for_mantra(mantra_text);
                    if !has_any {
                        errors.push(format!(
                            "{}:{}: mantra not found in self: ^{}^",
                            bhasya.file,
                            bhasya.line,
                            truncate(mantra_text, 30)
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
                    truncate(mantra_text, 30)
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
                let has_mula = external.mantras.contains_key(mantra_text);
                // check if any bhasya (mula or tyakta) exists
                let has_any_bhasya = external.has_any_bhasya_for_mantra(mantra_text);

                if !has_any_bhasya {
                    // no bhasya at all - error
                    errors.push(format!(
                        "{}:{}: mantra not found in shastra '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(mantra_text, 30)
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
                        truncate(mantra_text, 30)
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
fn check_khandita(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // cache parsed external shastras
    let mut shastra_repos: HashMap<String, Option<Repository>> = HashMap::new();

    // find all mula mantras in bhasyas with Khandita kind
    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        if let BhasyaKind::Khandita(ref shastra_name) = bhasya.kind {
            // resolve shastra name to path via shastra.json
            let Some(shastra_path) = repo.shastra_config.aliases.get(shastra_name) else {
                errors.push(format!(
                    "{}:{}: undefined shastra '{}' for khandita ^{}^",
                    bhasya.file,
                    bhasya.line,
                    shastra_name,
                    truncate(mantra_text, 30)
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
                let has_any_bhasya = external.has_any_bhasya_for_mantra(mantra_text);

                if !has_any_bhasya {
                    // no bhasya at all - error: can't refute what doesn't exist
                    errors.push(format!(
                        "{}:{}: cannot khandita non-existent bhasya from '{}': ^{}^",
                        bhasya.file,
                        bhasya.line,
                        shastra_name,
                        truncate(mantra_text, 30)
                    ));
                    continue;
                }
                // note: if source already tyakta'd it, that's fine - our khandita may have
                // contributed to that decision, so we keep it without warning
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

/// Check for unresolved conflicts between shastras I follow
/// If shastra X khandits a bhasya and shastra Y uddhrits it, I must take a position
fn check_unresolved_shastra_conflicts(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // collect my positions: (mantra_text, source_shastra) -> "khandita" | "uddhrit"
    let mut my_positions: HashMap<(String, String), &str> = HashMap::new();
    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        match &bhasya.kind {
            BhasyaKind::Khandita(shastra) => {
                my_positions.insert((mantra_text.to_string(), shastra.clone()), "khandita");
            }
            BhasyaKind::Uddhrit(shastra) => {
                my_positions.insert((mantra_text.to_string(), shastra.clone()), "uddhrit");
            }
            _ => {}
        }
    }

    // load all shastras I follow and collect their positions
    // key: (mantra_text, source_shastra) -> Vec<(follower_shastra, position)>
    let mut external_positions: HashMap<(String, String), Vec<(String, &str)>> = HashMap::new();

    for (shastra_name, shastra_path) in &repo.shastra_config.aliases {
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

        if let Ok(external) = Repository::parse(path) {
            for (ext_mantra_text, ext_bhasya) in external.mula_mantras_with_bhasyas() {
                match &ext_bhasya.kind {
                    BhasyaKind::Khandita(source) => {
                        let key = (ext_mantra_text.to_string(), source.clone());
                        external_positions
                            .entry(key)
                            .or_default()
                            .push((shastra_name.clone(), "khandita"));
                    }
                    BhasyaKind::Uddhrit(source) => {
                        let key = (ext_mantra_text.to_string(), source.clone());
                        external_positions
                            .entry(key)
                            .or_default()
                            .push((shastra_name.clone(), "uddhrit"));
                    }
                    _ => {}
                }
            }
        }
    }

    // find conflicts: same (mantra, source) has both khandita and uddhrit
    for (key, positions) in &external_positions {
        let has_khandita = positions.iter().any(|(_, pos)| *pos == "khandita");
        let has_uddhrit = positions.iter().any(|(_, pos)| *pos == "uddhrit");

        if has_khandita && has_uddhrit {
            // there's a conflict - check if I've resolved it
            if !my_positions.contains_key(key) {
                let khandita_by: Vec<_> = positions
                    .iter()
                    .filter(|(_, pos)| *pos == "khandita")
                    .map(|(s, _)| s.as_str())
                    .collect();
                let uddhrit_by: Vec<_> = positions
                    .iter()
                    .filter(|(_, pos)| *pos == "uddhrit")
                    .map(|(s, _)| s.as_str())
                    .collect();

                errors.push(format!(
                    "unresolved conflict for ^{}^ from '{}': khandita by [{}], uddhrit by [{}] - add your own khandita: or shastra: to resolve",
                    truncate(&key.0, 30),
                    key.1,
                    khandita_by.join(", "),
                    uddhrit_by.join(", ")
                ));
            }
        }
    }

    errors
}

/// Check that same bhasya is not both khandita and uddhrit from same shastra
/// If you refute a bhasya, you must refute it consistently - no quoting it elsewhere
fn check_khandita_uddhrit_conflicts(repo: &Repository) -> Vec<String> {
    let mut errors = Vec::new();

    // collect all khandita: (mantra_text, shastra) -> (file, line)
    let mut khandita_refs: HashMap<(String, String), (String, usize)> = HashMap::new();
    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        if let BhasyaKind::Khandita(ref shastra) = bhasya.kind {
            khandita_refs.insert(
                (mantra_text.to_string(), shastra.clone()),
                (bhasya.file.clone(), bhasya.line),
            );
        }
    }

    // check if any uddhrit matches a khandita
    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        if let BhasyaKind::Uddhrit(ref shastra) = bhasya.kind {
            let key = (mantra_text.to_string(), shastra.clone());
            if let Some((khandita_file, khandita_line)) = khandita_refs.get(&key) {
                errors.push(format!(
                    "{}:{}: cannot uddhrit ^{}^ from '{}' - already khandita at {}:{}",
                    bhasya.file,
                    bhasya.line,
                    truncate(mantra_text, 30),
                    shastra,
                    khandita_file,
                    khandita_line
                ));
            }
        }
    }

    errors
}

/// Check for duplicate bhasyas - same mantra in same paragraph content must be unique
/// Returns: Vec<(mantra_text, paragraph, locations)>
fn check_duplicate_bhasyas(repo: &Repository) -> Vec<(String, String, Vec<(String, usize)>)> {
    // key: (mantra_text, paragraph) -> list of (file, line)
    let mut occurrences: HashMap<(String, String), Vec<(String, usize, String)>> = HashMap::new();

    for (mantra_text, bhasya) in repo.mula_mantras_with_bhasyas() {
        // skip non-Mula bhasyas - duplicates allowed for uddhrit/khandita
        if !matches!(bhasya.kind, BhasyaKind::Mula) {
            continue;
        }

        // normalize paragraph for comparison (trim whitespace)
        let normalized_para = bhasya.paragraph.lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");

        let key = (mantra_text.to_string(), normalized_para);
        occurrences
            .entry(key)
            .or_default()
            .push((bhasya.file.clone(), bhasya.line, bhasya.paragraph.clone()));
    }

    // collect only those with more than one occurrence
    occurrences
        .into_iter()
        .filter(|(_, locs)| locs.len() > 1)
        .map(|((mantra, _commentary), locs)| {
            // use the paragraph from the first occurrence for display
            let paragraph = locs.first().map(|(_, _, p)| p.to_string()).unwrap_or_default();
            (
                mantra.to_string(),
                paragraph,
                locs.into_iter().map(|(f, l, _)| (f.to_string(), l)).collect(),
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
