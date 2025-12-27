use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{Canon, CanonSearchResult};
use std::path::Path;

pub fn run(path: &Path, kosha: Option<String>, count: usize) -> Result<(), String> {
    let repo = Repository::parse(path)?;
    let repo_root = find_repo_root(path).ok_or("could not find repository root")?;

    // load our canon
    let our_canon = match Canon::find(&repo_root) {
        CanonSearchResult::Found(c) => c,
        CanonSearchResult::NotFound => Canon::default(),
        CanonSearchResult::Multiple(paths) => {
            return Err(format!(
                "multiple canon.md files found:\n  {}",
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

    // get configured koshas
    let aliases = &repo.kosha_config.aliases;

    match kosha {
        Some(kosha_name) => {
            if aliases.is_empty() {
                return Err("no koshas configured in .vyasa/kosha.md".to_string());
            }
            // study a specific kosha
            study_kosha(&kosha_name, &repo, &our_canon, count)
        }
        None => {
            if aliases.is_empty() {
                println!("no koshas configured in .vyasa/kosha.md");
                return Ok(());
            }
            // show stats for all koshas
            show_all_stats(&repo, &our_canon)
        }
    }
}

fn study_kosha(
    kosha_name: &str,
    repo: &Repository,
    our_canon: &Canon,
    count: usize,
) -> Result<(), String> {
    // check if alias exists
    let alias = repo
        .kosha_config
        .aliases
        .iter()
        .find(|a| a.alias == kosha_name);

    if alias.is_none() {
        return Err(format!(
            "kosha '{}' not found. configured koshas: {}",
            kosha_name,
            repo.kosha_config
                .aliases
                .iter()
                .map(|a| a.alias.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // resolve and load the kosha's canon
    let resolved = repo.resolve_kosha_path(kosha_name).ok_or(format!(
        "could not resolve kosha '{}' - check kosha.local.md",
        kosha_name
    ))?;

    let kosha_path = Path::new(&resolved);
    if !kosha_path.exists() {
        return Err(format!("kosha path does not exist: {}", resolved));
    }

    let kosha_canon = match Canon::find(kosha_path) {
        CanonSearchResult::Found(c) => c,
        CanonSearchResult::NotFound => {
            return Err(format!("kosha '{}' has no canon.md", kosha_name));
        }
        CanonSearchResult::Multiple(paths) => {
            return Err(format!(
                "kosha '{}' has multiple canon.md files:\n  {}",
                kosha_name,
                paths.join("\n  ")
            ));
        }
        CanonSearchResult::Invalid { path, errors } => {
            return Err(format!(
                "kosha '{}' has invalid canon.md at {}:\n  {}",
                kosha_name,
                path,
                errors.join("\n  ")
            ));
        }
    };

    // find mantras in kosha's canon that are not in our canon
    let mut missing: Vec<_> = kosha_canon
        .entries
        .iter()
        .filter(|(mantra, _)| our_canon.get(mantra).is_none())
        .collect();

    // sort by mantra text for consistent ordering
    missing.sort_by(|a, b| a.0.cmp(b.0));

    if missing.is_empty() {
        println!("you are up to date with '{}'!", kosha_name);
        return Ok(());
    }

    println!("you are {} items behind:\n", missing.len());

    // show first `count` items
    for (mantra, entry) in missing.iter().take(count) {
        if !entry.source_file.is_empty() {
            println!("{}:", entry.source_file);
        }
        println!("> **^{}^**", mantra);
        if !entry.commentary.is_empty() {
            // indent commentary
            for line in entry.commentary.lines() {
                println!("> {}", line);
            }
        }
        println!();
    }

    if missing.len() > count {
        println!("... and {} more", missing.len() - count);
    }

    Ok(())
}

fn show_all_stats(repo: &Repository, our_canon: &Canon) -> Result<(), String> {
    let mut results: Vec<(String, usize, Option<String>)> = Vec::new();

    for alias in &repo.kosha_config.aliases {
        let kosha_name = &alias.alias;

        // try to resolve and load the kosha's canon
        let count = match repo.resolve_kosha_path(kosha_name) {
            Some(resolved) => {
                let kosha_path = Path::new(&resolved);
                if !kosha_path.exists() {
                    results.push((kosha_name.clone(), 0, Some("path not found".to_string())));
                    continue;
                }

                match Canon::find(kosha_path) {
                    CanonSearchResult::Found(kosha_canon) => {
                        // count mantras not in our canon
                        kosha_canon
                            .entries
                            .keys()
                            .filter(|m| our_canon.get(m).is_none())
                            .count()
                    }
                    CanonSearchResult::NotFound => {
                        results.push((kosha_name.clone(), 0, Some("no canon.md".to_string())));
                        continue;
                    }
                    _ => {
                        results.push((kosha_name.clone(), 0, Some("invalid canon".to_string())));
                        continue;
                    }
                }
            }
            None => {
                results.push((
                    kosha_name.clone(),
                    0,
                    Some("not resolved (check kosha.local.md)".to_string()),
                ));
                continue;
            }
        };

        results.push((kosha_name.clone(), count, None));
    }

    // sort by count descending
    results.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, count, error) in &results {
        if let Some(err) = error {
            println!("{}: ({})", name, err);
        } else if *count == 0 {
            println!("{}: up to date", name);
        } else {
            println!("{}: {}", name, count);
        }
    }

    Ok(())
}
