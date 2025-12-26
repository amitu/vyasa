use crate::parser::Repository;
use std::collections::HashMap;
use std::path::Path;

// [vyasa values cli can query placeholder in file/directory, and filter mantras or even keys]
// [vyasa mantra filename is optional, can be folder or pattern]
pub fn run(
    path: &str,
    mantra_filter: Option<String>,
    key_filter: Option<String>,
) -> Result<(), String> {
    let path = Path::new(path);
    let repo = Repository::parse(path)?;
    let all_values = repo.extract_placeholder_values();

    // parse mantra filter - strip [] if present
    let mantra_filter = mantra_filter.map(|m| {
        let m = m.trim();
        if m.starts_with('[') && m.ends_with(']') {
            m[1..m.len() - 1].to_string()
        } else {
            m.to_string()
        }
    });

    // apply filters
    let filtered: Vec<_> = all_values
        .into_iter()
        .filter(|v| {
            // mantra filter: exact match on template
            if let Some(ref m) = mantra_filter {
                if v.template != *m {
                    return false;
                }
            }
            if let Some(ref k) = key_filter {
                if v.key != *k {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("no placeholder values found");
        return Ok(());
    }

    // group by template, then by key
    let mut by_template: HashMap<String, HashMap<String, Vec<(String, String, usize)>>> =
        HashMap::new();

    for v in filtered {
        by_template
            .entry(v.template.clone())
            .or_default()
            .entry(v.key.clone())
            .or_default()
            .push((v.value, v.file, v.line));
    }

    // print results
    for (template, keys) in by_template.iter() {
        println!("[{}]", template);
        println!();

        for (key, values) in keys.iter() {
            // deduplicate values, keeping first occurrence
            let mut seen: HashMap<String, (String, usize)> = HashMap::new();
            for (value, file, line) in values {
                seen.entry(value.clone())
                    .or_insert_with(|| (file.clone(), *line));
            }

            println!("  {{{}}}: {} unique values", key, seen.len());
            let mut sorted: Vec<_> = seen.into_iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));

            for (value, (file, line)) in sorted {
                println!("    {}={} ({}:{})", key, value, file, line);
            }
            println!();
        }
    }

    Ok(())
}
