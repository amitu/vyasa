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
    // [mantra with kosha references look like {mantra}@{kosha-name}]
    pub kosha: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpacingViolation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

// [kosha-alias {kosha-alias}: {kosha-value}]
#[derive(Debug, Clone)]
pub struct KoshaAlias {
    pub alias: String,
    pub value: String,
}

// [kosha-dir {kosha-alias}: {folder-name}]
#[derive(Debug, Clone)]
pub struct KoshaDir {
    pub alias: String,
    pub folder: String,
}

#[derive(Debug, Default)]
pub struct KoshaConfig {
    pub aliases: Vec<KoshaAlias>,
    pub local_dirs: Vec<KoshaDir>,
}

#[derive(Debug, Default)]
pub struct Repository {
    pub mantras: HashMap<String, Mantra>,
    pub references: Vec<Reference>,
    pub spacing_violations: Vec<SpacingViolation>,
    pub kosha_config: KoshaConfig,
}

impl Repository {
    // [.vyasa and .md are both allowed]
    pub fn parse(path: &Path) -> Result<Self, String> {
        let mut repo = Repository::default();

        // [.vyasa/kosha.vyasa contains mantra with kosha references]
        // find repository root and load kosha config
        let repo_root = find_repo_root(path);
        if let Some(root) = &repo_root {
            repo.kosha_config = load_kosha_config(root);
        }

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

    pub fn resolve_kosha_path(&self, alias: &str) -> Option<String> {
        // first check local overrides
        for local in &self.kosha_config.local_dirs {
            if local.alias == alias {
                return Some(local.folder.clone());
            }
        }
        // then check aliases
        for kosha in &self.kosha_config.aliases {
            if kosha.alias == alias {
                return Some(kosha.value.clone());
            }
        }
        None
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
    let mut in_backtick = false;

    while let Some(c) = chars.next() {
        // skip content inside backticks (inline code)
        if c == '`' {
            in_backtick = !in_backtick;
            continue;
        }
        if in_backtick {
            continue;
        }

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
                // [mantra with kosha references look like {mantra}@{kosha-name}]
                // check for @kosha suffix after the closing ]
                let mut kosha = None;
                if chars.peek() == Some(&'@') {
                    chars.next(); // consume @
                    let mut kosha_name = String::new();
                    for c in chars.by_ref() {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            kosha_name.push(c);
                        } else {
                            break;
                        }
                    }
                    if !kosha_name.is_empty() {
                        kosha = Some(kosha_name);
                    }
                }

                repo.references.push(Reference {
                    mantra_text: ref_text,
                    file: file_name.to_string(),
                    line: line_num,
                    matched_template: None,
                    kosha,
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
        for (template_text, template_regex) in &templates {
            if let Some(tr) = template_regex {
                if matches_template(tr, &reference.mantra_text) {
                    reference.matched_template = Some(template_text.clone());
                    break;
                }
            }
        }
    }
}

// [template placeholders can include example values as {name=example}]
// extracts the variable name from a placeholder, stripping any =example suffix
fn extract_placeholder_name(placeholder_content: &str) -> &str {
    if let Some(eq_pos) = placeholder_content.find('=') {
        &placeholder_content[..eq_pos]
    } else {
        placeholder_content
    }
}

// extracts the example value from a placeholder like {name=example}, returns None if no example
fn extract_example_value(placeholder_content: &str) -> Option<&str> {
    if let Some(eq_pos) = placeholder_content.find('=') {
        Some(&placeholder_content[eq_pos + 1..])
    } else {
        None
    }
}

// Stores info about each capture group in the template regex
#[derive(Debug, Clone)]
struct CaptureInfo {
    placeholder_idx: usize,  // which placeholder this capture belongs to
    full_placeholder: String, // e.g., "{employee=amitu}"
}

// Result of building a template regex - includes the regex and capture group info
#[derive(Debug)]
struct TemplateRegex {
    regex: Regex,
    captures: Vec<CaptureInfo>, // info for each capture group (1-indexed in regex)
}

// [when referencing a mantra with placeholder you can either use the reference with placeholder or (even partially) instantiated]
// [example values appear literally in the mantra text]
fn build_template_regex(template: &str) -> Option<TemplateRegex> {
    // first pass: collect all placeholders and their example values
    let mut placeholders: Vec<(usize, usize, String, Option<String>)> = Vec::new(); // (start, end, full_placeholder, example_value)

    for (start, _) in template.match_indices('{') {
        if let Some(end_offset) = template[start..].find('}') {
            let end = start + end_offset;
            let content = &template[start + 1..end];
            let full_placeholder = template[start..=end].to_string();
            let example = extract_example_value(content).map(|s| s.to_string());
            placeholders.push((start, end, full_placeholder, example));
        }
    }

    // collect all positions that need capture groups
    // each position stores: (start, end, placeholder_idx, full_placeholder)
    let mut captures: Vec<(usize, usize, usize, String)> = Vec::new();

    // add placeholder positions
    for (idx, (start, end, full_placeholder, _)) in placeholders.iter().enumerate() {
        captures.push((*start, *end, idx, full_placeholder.clone()));
    }

    // add example value literal occurrences (outside of placeholders)
    for (placeholder_idx, (_, _, full_placeholder, example)) in placeholders.iter().enumerate() {
        if let Some(ex) = example {
            // find all occurrences of the example value in template
            for (pos, _) in template.match_indices(ex.as_str()) {
                // skip if this occurrence is inside a placeholder
                let inside_placeholder = placeholders.iter().any(|(s, e, _, _)| pos >= *s && pos <= *e);
                if !inside_placeholder {
                    captures.push((pos, pos + ex.len() - 1, placeholder_idx, full_placeholder.clone()));
                }
            }
        }
    }

    // sort by position
    captures.sort_by_key(|(start, _, _, _)| *start);

    // build the regex pattern - each position becomes a capture group
    let mut pattern = String::from("^");
    let mut last_end = 0;
    let mut capture_info: Vec<CaptureInfo> = Vec::new();

    for (start, end, placeholder_idx, full_placeholder) in &captures {
        // add escaped literal part before this capture
        if *start > last_end {
            pattern.push_str(&regex::escape(&template[last_end..*start]));
        }

        // capture group that matches any value OR the literal placeholder
        pattern.push_str(&format!("(.+?|{})", regex::escape(full_placeholder)));
        capture_info.push(CaptureInfo {
            placeholder_idx: *placeholder_idx,
            full_placeholder: full_placeholder.clone(),
        });

        last_end = end + 1;
    }

    // add remaining literal part
    if last_end < template.len() {
        pattern.push_str(&regex::escape(&template[last_end..]));
    }
    pattern.push('$');

    Regex::new(&pattern).ok().map(|regex| TemplateRegex { regex, captures: capture_info })
}

// Check if a reference matches a template, verifying consistent placeholder values
fn matches_template(template_regex: &TemplateRegex, reference: &str) -> bool {
    if let Some(caps) = template_regex.regex.captures(reference) {
        // verify that all captures for the same placeholder have the same value
        let mut placeholder_values: std::collections::HashMap<usize, &str> = std::collections::HashMap::new();

        for (i, info) in template_regex.captures.iter().enumerate() {
            if let Some(m) = caps.get(i + 1) {
                let value = m.as_str();
                // skip if it's the literal placeholder
                if value == info.full_placeholder {
                    continue;
                }
                // check consistency
                if let Some(&existing) = placeholder_values.get(&info.placeholder_idx) {
                    if existing != value {
                        return false; // inconsistent values for same placeholder
                    }
                } else {
                    placeholder_values.insert(info.placeholder_idx, value);
                }
            }
        }
        true
    } else {
        false
    }
}

fn extract_values_from_reference(template: &str, reference: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();

    // Build the template regex with capture info
    let Some(template_regex) = build_template_regex(template) else {
        return results;
    };

    // collect placeholder info (variable name, full content) for each placeholder
    let mut placeholder_info: Vec<(String, String)> = Vec::new();
    for (start, _) in template.match_indices('{') {
        if let Some(end) = template[start..].find('}') {
            let end = start + end;
            let full_content = &template[start + 1..end];
            let var_name = extract_placeholder_name(full_content);
            placeholder_info.push((var_name.to_string(), full_content.to_string()));
        }
    }

    // capture values from reference
    if let Some(caps) = template_regex.regex.captures(reference) {
        // track which placeholder indices we've already extracted
        let mut extracted: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (i, info) in template_regex.captures.iter().enumerate() {
            // only extract once per placeholder
            if extracted.contains(&info.placeholder_idx) {
                continue;
            }

            if let Some(m) = caps.get(i + 1) {
                let value = m.as_str().to_string();
                // skip if value is the placeholder itself
                if value == info.full_placeholder {
                    continue;
                }

                // get the variable name from placeholder_info
                if let Some((var_name, _)) = placeholder_info.get(info.placeholder_idx) {
                    results.push((var_name.clone(), value));
                    extracted.insert(info.placeholder_idx);
                }
            }
        }
    }

    results
}

fn find_repo_root(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = if path.is_file() {
        path.parent().map(|p| p.to_path_buf())
    } else {
        Some(path.to_path_buf())
    };

    while let Some(dir) = current {
        if dir.join(".vyasa").is_dir() || dir.join(".git").is_dir() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    None
}

// [.vyasa/kosha.vyasa contains mantra with kosha references]
fn load_kosha_config(repo_root: &Path) -> KoshaConfig {
    let mut config = KoshaConfig::default();

    // load main kosha config
    let kosha_file = repo_root.join(".vyasa/kosha.vyasa");
    if let Ok(content) = fs::read_to_string(&kosha_file) {
        parse_kosha_aliases(&content, &mut config);
    }

    // [.vyasa/kosha.local.vyasa, to be .gitignored stores local folders for each referenced kosha]
    let local_file = repo_root.join(".vyasa/kosha.local.vyasa");
    if let Ok(content) = fs::read_to_string(&local_file) {
        parse_kosha_local(&content, &mut config);
    }

    config
}

fn parse_kosha_aliases(content: &str, config: &mut KoshaConfig) {
    // look for references matching [kosha-alias {alias}: {value}] pattern
    let alias_pattern = Regex::new(r"\[kosha-alias\s+([^:]+):\s*([^\]]+)\]").ok();

    if let Some(re) = alias_pattern {
        for cap in re.captures_iter(content) {
            if let (Some(alias), Some(value)) = (cap.get(1), cap.get(2)) {
                config.aliases.push(KoshaAlias {
                    alias: alias.as_str().trim().to_string(),
                    value: value.as_str().trim().to_string(),
                });
            }
        }
    }
}

fn parse_kosha_local(content: &str, config: &mut KoshaConfig) {
    // look for references matching [kosha-dir {alias}: {folder}] pattern
    let dir_pattern = Regex::new(r"\[kosha-dir\s+([^:]+):\s*([^\]]+)\]").ok();

    if let Some(re) = dir_pattern {
        for cap in re.captures_iter(content) {
            if let (Some(alias), Some(folder)) = (cap.get(1), cap.get(2)) {
                config.local_dirs.push(KoshaDir {
                    alias: alias.as_str().trim().to_string(),
                    folder: folder.as_str().trim().to_string(),
                });
            }
        }
    }
}
