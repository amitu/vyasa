use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Mantra {
    pub text: String,
    pub file: String,
    pub line: usize,
    pub has_explanation: bool,
    pub is_template: bool,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub mantra_text: String,
    pub file: String,
    pub line: usize,
    pub matched_template: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpacingViolation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Repository {
    pub mantras: HashMap<String, Mantra>,
    pub references: Vec<Reference>,
    pub spacing_violations: Vec<SpacingViolation>,
}

impl Repository {
    // [.vyasa and .md are both allowed]
    pub fn parse(path: &Path) -> Result<Self, String> {
        let mut repo = Repository::default();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // skip hidden files and directories (but not . or ..)
            if file_path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s.starts_with('.') && s.len() > 2
            }) {
                continue;
            }

            // [.vyasa and .md are both allowed]
            let ext = file_path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("vyasa") | Some("md")) {
                continue;
            }

            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let file_name = file_path.to_string_lossy().to_string();
            parse_file(&content, &file_name, &mut repo);
        }

        // match references to template mantras
        resolve_template_references(&mut repo);

        Ok(repo)
    }

    pub fn unreferenced_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| {
                !self.references.iter().any(|r| {
                    r.mantra_text == m.text
                        || r.matched_template.as_ref() == Some(&m.text)
                })
            })
            .collect()
    }

    pub fn unexplained_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| !m.has_explanation)
            .collect()
    }

    pub fn reference_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for reference in &self.references {
            // count against template if matched, otherwise the literal text
            let key = reference
                .matched_template
                .as_ref()
                .unwrap_or(&reference.mantra_text);
            *counts.entry(key.clone()).or_insert(0) += 1;
        }
        counts
    }

    // [vyasa values cli can query placeholder in file/directory, and filter mantras or even keys]
    pub fn extract_placeholder_values(&self) -> Vec<PlaceholderValue> {
        let mut values = Vec::new();

        for reference in &self.references {
            if let Some(template_text) = &reference.matched_template {
                if let Some(mantra) = self.mantras.get(template_text) {
                    if mantra.is_template {
                        let extracted = extract_values_from_reference(
                            template_text,
                            &reference.mantra_text,
                        );
                        for (key, value) in extracted {
                            values.push(PlaceholderValue {
                                template: template_text.clone(),
                                key,
                                value,
                                file: reference.file.clone(),
                                line: reference.line,
                            });
                        }
                    }
                }
            }
        }

        values
    }
}

#[derive(Debug, Clone)]
pub struct PlaceholderValue {
    pub template: String,
    pub key: String,
    pub value: String,
    pub file: String,
    pub line: usize,
}

fn parse_file(content: &str, file_name: &str, repo: &mut Repository) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut i = 0;
    let mut in_code_block = false;

    while i < lines.len() {
        let line = lines[i].trim();

        // skip markdown code blocks (``` delimited)
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            i += 1;
            continue;
        }

        if in_code_block {
            i += 1;
            continue;
        }

        // check for mantra start
        if line == "--" {
            // [before / after -- must contain at least one empty line, unless at start/end of file]
            check_spacing_before(&lines, i, file_name, repo);

            if let Some((mantra, end_line, has_explanation)) =
                parse_mantra(&lines, i, file_name)
            {
                // check spacing after closing --
                check_spacing_after(&lines, end_line, total_lines, file_name, repo);

                repo.mantras.insert(mantra.text.clone(), mantra);
                i = end_line + 1;

                // skip explanation text until next mantra or end
                if has_explanation {
                    while i < lines.len() && lines[i].trim() != "--" {
                        // also skip code blocks in explanations
                        if lines[i].trim().starts_with("```") {
                            in_code_block = !in_code_block;
                            i += 1;
                            continue;
                        }
                        if !in_code_block {
                            parse_references(lines[i], file_name, i + 1, repo);
                        }
                        i += 1;
                    }
                }
                continue;
            }
        }

        parse_references(lines[i], file_name, i + 1, repo);
        i += 1;
    }
}

// [before / after -- must contain at least one empty line, unless at start/end of file]
fn check_spacing_before(
    lines: &[&str],
    line_idx: usize,
    file_name: &str,
    repo: &mut Repository,
) {
    if line_idx == 0 {
        return; // start of file, no spacing needed
    }

    let prev_line = lines[line_idx - 1].trim();
    // allow if previous line is empty or another -- (consecutive mantras ok)
    if !prev_line.is_empty() && prev_line != "--" {
        repo.spacing_violations.push(SpacingViolation {
            file: file_name.to_string(),
            line: line_idx + 1,
            message: "missing empty line before --".to_string(),
        });
    }
}

fn check_spacing_after(
    lines: &[&str],
    closing_line: usize,
    total_lines: usize,
    file_name: &str,
    repo: &mut Repository,
) {
    if closing_line + 1 >= total_lines {
        return; // end of file, no spacing needed
    }

    let next_line = lines[closing_line + 1].trim();
    // allow if next line is empty or another -- (consecutive mantras ok)
    if !next_line.is_empty() && next_line != "--" {
        repo.spacing_violations.push(SpacingViolation {
            file: file_name.to_string(),
            line: closing_line + 2,
            message: "missing empty line after --".to_string(),
        });
    }
}

fn parse_mantra(
    lines: &[&str],
    start: usize,
    file_name: &str,
) -> Option<(Mantra, usize, bool)> {
    let mut i = start + 1;
    let mut mantra_lines = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();
        if line == "--" {
            break;
        }
        mantra_lines.push(lines[i]);
        i += 1;
    }

    if i >= lines.len() {
        return None;
    }

    let mantra_text = mantra_lines.join("\n").trim().to_string();
    if mantra_text.is_empty() {
        return None;
    }

    // [mantra specifications can use string formatting templates for mantra with value]
    let is_template = mantra_text.contains('{') && mantra_text.contains('}');

    // check if there's explanation after the closing --
    let mut has_explanation = false;
    let mut j = i + 1;
    while j < lines.len() {
        let next_line = lines[j].trim();
        if next_line == "--" {
            break;
        }
        if !next_line.is_empty() {
            has_explanation = true;
            break;
        }
        j += 1;
    }

    Some((
        Mantra {
            text: mantra_text,
            file: file_name.to_string(),
            line: start + 1,
            has_explanation,
            is_template,
        },
        i,
        has_explanation,
    ))
}

fn parse_references(line: &str, file_name: &str, line_num: usize, repo: &mut Repository) {
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut ref_text = String::new();
            let mut depth = 1;

            for c in chars.by_ref() {
                if c == '[' {
                    depth += 1;
                    ref_text.push(c);
                } else if c == ']' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    ref_text.push(c);
                } else {
                    ref_text.push(c);
                }
            }

            let ref_text = ref_text.trim().to_string();
            if !ref_text.is_empty() && depth == 0 {
                repo.references.push(Reference {
                    mantra_text: ref_text,
                    file: file_name.to_string(),
                    line: line_num,
                    matched_template: None,
                });
            }
        }
    }
}

// [mantra specifications can use string formatting templates for mantra with value]
fn resolve_template_references(repo: &mut Repository) {
    let templates: Vec<_> = repo
        .mantras
        .values()
        .filter(|m| m.is_template)
        .map(|m| (m.text.clone(), build_template_regex(&m.text)))
        .collect();

    for reference in &mut repo.references {
        // skip if already an exact match
        if repo.mantras.contains_key(&reference.mantra_text) {
            continue;
        }

        // try to match against templates
        for (template_text, regex) in &templates {
            if let Some(re) = regex {
                if re.is_match(&reference.mantra_text) {
                    reference.matched_template = Some(template_text.clone());
                    break;
                }
            }
        }
    }
}

// [when referencing a mantra with placeholder you can either use the reference with placeholder or (even partially) instantiated]
fn build_template_regex(template: &str) -> Option<Regex> {
    // escape regex special chars, then replace {name} with pattern that matches
    // either the literal placeholder OR any substituted value
    let mut pattern = String::from("^");
    let mut last_end = 0;

    for (start, _) in template.match_indices('{') {
        if let Some(end) = template[start..].find('}') {
            let end = start + end;
            // add escaped literal part before this placeholder
            pattern.push_str(&regex::escape(&template[last_end..start]));
            // extract the placeholder including braces
            let placeholder = &template[start..=end];
            // match either: any value (.+) OR the literal placeholder itself
            pattern.push_str(&format!("(.+|{})", regex::escape(placeholder)));
            last_end = end + 1;
        }
    }

    // add remaining literal part
    pattern.push_str(&regex::escape(&template[last_end..]));
    pattern.push('$');

    Regex::new(&pattern).ok()
}

fn extract_values_from_reference(template: &str, reference: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut placeholder_names = Vec::new();

    // first, collect placeholder names in order
    for (start, _) in template.match_indices('{') {
        if let Some(end) = template[start..].find('}') {
            let end = start + end;
            let name = &template[start + 1..end];
            placeholder_names.push(name.to_string());
        }
    }

    // build regex with named capture groups
    let mut pattern = String::from("^");
    let mut last_end = 0;

    for (start, _) in template.match_indices('{') {
        if let Some(end) = template[start..].find('}') {
            let end = start + end;
            pattern.push_str(&regex::escape(&template[last_end..start]));
            let placeholder = &template[start..=end];
            // capture group that matches value or literal placeholder
            pattern.push_str(&format!("(.+|{})", regex::escape(placeholder)));
            last_end = end + 1;
        }
    }
    pattern.push_str(&regex::escape(&template[last_end..]));
    pattern.push('$');

    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(reference) {
            for (i, name) in placeholder_names.iter().enumerate() {
                if let Some(m) = caps.get(i + 1) {
                    let value = m.as_str().to_string();
                    // skip if value is the placeholder itself
                    let placeholder = format!("{{{}}}", name);
                    if value != placeholder {
                        results.push((name.clone(), value));
                    }
                }
            }
        }
    }

    results
}
