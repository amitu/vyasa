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

// ^mantra^@kosha provides commentary on external mantra
#[derive(Debug, Clone)]
pub struct ExternalCommentary {
    pub mantra_text: String,
    pub kosha: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub mantra_text: String,
    pub file: String,
    pub line: usize,
    pub matched_template: Option<String>,
    // ~mantra~@kosha-name for external references
    pub kosha: Option<String>,
}

// ^kosha-alias {kosha-alias}: {kosha-value}^
#[derive(Debug, Clone)]
pub struct KoshaAlias {
    pub alias: String,
    pub value: String,
}

// ^kosha-dir {kosha-alias}: {folder-name}^
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
    pub external_commentaries: Vec<ExternalCommentary>,
    pub kosha_config: KoshaConfig,
}

impl Repository {
    // ^.vyasa and .md are both allowed^
    pub fn parse(path: &Path) -> Result<Self, String> {
        let mut repo = Repository::default();

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

            // ^.vyasa and .md are both allowed^
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

        // determine which mantras have explanations (commentary in same paragraph)
        mark_explained_mantras(&mut repo);

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

// ^mantras should use inline syntax not block because they are meant to be short^
fn parse_file(content: &str, file_name: &str, repo: &mut Repository) {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_code_block = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        // skip markdown code blocks (``` delimited)
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            continue;
        }

        // parse both mantra definitions (^...^) and references (~...~) from this line
        parse_line(line, file_name, line_num, repo);
    }
}

// Parse a single line for mantra definitions and references
fn parse_line(line: &str, file_name: &str, line_num: usize, repo: &mut Repository) {
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

        // ^mantra definition^ or ^mantra^@kosha (external commentary)
        if c == '^' {
            let mut mantra_text = String::new();

            for c in chars.by_ref() {
                if c == '^' {
                    break;
                }
                mantra_text.push(c);
            }

            let mantra_text = mantra_text.trim().to_string();
            if !mantra_text.is_empty() {
                // check for @kosha suffix (external commentary)
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
                        // this is external commentary, not a local mantra definition
                        repo.external_commentaries.push(ExternalCommentary {
                            mantra_text,
                            kosha: kosha_name,
                            file: file_name.to_string(),
                            line: line_num,
                        });
                    }
                } else {
                    // local mantra definition
                    let is_template = mantra_text.contains('{') && mantra_text.contains('}');
                    let has_explanation = line_has_commentary(line, &mantra_text);

                    repo.mantras.insert(mantra_text.clone(), Mantra {
                        text: mantra_text,
                        file: file_name.to_string(),
                        line: line_num,
                        has_explanation,
                        is_template,
                    });
                }
            }
        }

        // ~reference~ with optional @kosha
        if c == '~' {
            let mut ref_text = String::new();

            for c in chars.by_ref() {
                if c == '~' {
                    break;
                }
                ref_text.push(c);
            }

            let ref_text = ref_text.trim().to_string();
            if !ref_text.is_empty() {
                // check for @kosha suffix
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

// Check if a line has text beyond just the mantra definition
fn line_has_commentary(line: &str, mantra_text: &str) -> bool {
    // remove the mantra definition from the line and check if there's other text
    let pattern = format!("^{}^", mantra_text);
    let remaining = line.replace(&pattern, "");
    let trimmed = remaining.trim();

    // has commentary if there's non-empty text that isn't just punctuation
    !trimmed.is_empty() && trimmed.chars().any(|c| c.is_alphanumeric())
}

// ^mantra commentary can be in same para^ - mark mantras as explained if they have nearby commentary
fn mark_explained_mantras(_repo: &mut Repository) {
    // For now, mantras are marked as explained during parsing if they have
    // text in the same line. A more sophisticated approach could look at
    // the surrounding paragraph, but inline commentary is the primary case.
    // The has_explanation field is already set in parse_line.
}

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

// ^template placeholders can include example values as {name=example}^
fn extract_placeholder_name(placeholder_content: &str) -> &str {
    if let Some(eq_pos) = placeholder_content.find('=') {
        &placeholder_content[..eq_pos]
    } else {
        placeholder_content
    }
}

fn extract_example_value(placeholder_content: &str) -> Option<&str> {
    if let Some(eq_pos) = placeholder_content.find('=') {
        Some(&placeholder_content[eq_pos + 1..])
    } else {
        None
    }
}

#[derive(Debug, Clone)]
struct CaptureInfo {
    placeholder_idx: usize,
    full_placeholder: String,
}

#[derive(Debug)]
struct TemplateRegex {
    regex: Regex,
    captures: Vec<CaptureInfo>,
}

fn build_template_regex(template: &str) -> Option<TemplateRegex> {
    let mut placeholders: Vec<(usize, usize, String, Option<String>)> = Vec::new();

    for (start, _) in template.match_indices('{') {
        if let Some(end_offset) = template[start..].find('}') {
            let end = start + end_offset;
            let content = &template[start + 1..end];
            let full_placeholder = template[start..=end].to_string();
            let example = extract_example_value(content).map(|s| s.to_string());
            placeholders.push((start, end, full_placeholder, example));
        }
    }

    let mut captures: Vec<(usize, usize, usize, String)> = Vec::new();

    for (idx, (start, end, full_placeholder, _)) in placeholders.iter().enumerate() {
        captures.push((*start, *end, idx, full_placeholder.clone()));
    }

    for (placeholder_idx, (_, _, full_placeholder, example)) in placeholders.iter().enumerate() {
        if let Some(ex) = example {
            for (pos, _) in template.match_indices(ex.as_str()) {
                let inside_placeholder = placeholders.iter().any(|(s, e, _, _)| pos >= *s && pos <= *e);
                if !inside_placeholder {
                    captures.push((pos, pos + ex.len() - 1, placeholder_idx, full_placeholder.clone()));
                }
            }
        }
    }

    captures.sort_by_key(|(start, _, _, _)| *start);

    let mut pattern = String::from("^");
    let mut last_end = 0;
    let mut capture_info: Vec<CaptureInfo> = Vec::new();

    for (start, end, placeholder_idx, full_placeholder) in &captures {
        if *start > last_end {
            pattern.push_str(&regex::escape(&template[last_end..*start]));
        }

        pattern.push_str(&format!("(.+?|{})", regex::escape(full_placeholder)));
        capture_info.push(CaptureInfo {
            placeholder_idx: *placeholder_idx,
            full_placeholder: full_placeholder.clone(),
        });

        last_end = end + 1;
    }

    if last_end < template.len() {
        pattern.push_str(&regex::escape(&template[last_end..]));
    }
    pattern.push('$');

    Regex::new(&pattern).ok().map(|regex| TemplateRegex { regex, captures: capture_info })
}

fn matches_template(template_regex: &TemplateRegex, reference: &str) -> bool {
    if let Some(caps) = template_regex.regex.captures(reference) {
        let mut placeholder_values: std::collections::HashMap<usize, &str> = std::collections::HashMap::new();

        for (i, info) in template_regex.captures.iter().enumerate() {
            if let Some(m) = caps.get(i + 1) {
                let value = m.as_str();
                if value == info.full_placeholder {
                    continue;
                }
                if let Some(&existing) = placeholder_values.get(&info.placeholder_idx) {
                    if existing != value {
                        return false;
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

    let Some(template_regex) = build_template_regex(template) else {
        return results;
    };

    let mut placeholder_info: Vec<(String, String)> = Vec::new();
    for (start, _) in template.match_indices('{') {
        if let Some(end) = template[start..].find('}') {
            let end = start + end;
            let full_content = &template[start + 1..end];
            let var_name = extract_placeholder_name(full_content);
            placeholder_info.push((var_name.to_string(), full_content.to_string()));
        }
    }

    if let Some(caps) = template_regex.regex.captures(reference) {
        let mut extracted: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (i, info) in template_regex.captures.iter().enumerate() {
            if extracted.contains(&info.placeholder_idx) {
                continue;
            }

            if let Some(m) = caps.get(i + 1) {
                let value = m.as_str().to_string();
                if value == info.full_placeholder {
                    continue;
                }

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

fn load_kosha_config(repo_root: &Path) -> KoshaConfig {
    let mut config = KoshaConfig::default();

    let kosha_file = repo_root.join(".vyasa/kosha.vyasa");
    if let Ok(content) = fs::read_to_string(&kosha_file) {
        parse_kosha_aliases(&content, &mut config);
    }

    let local_file = repo_root.join(".vyasa/kosha.local.vyasa");
    if let Ok(content) = fs::read_to_string(&local_file) {
        parse_kosha_local(&content, &mut config);
    }

    config
}

fn parse_kosha_aliases(content: &str, config: &mut KoshaConfig) {
    // look for ~kosha-alias {alias}: {value}~ pattern
    let alias_pattern = Regex::new(r"~kosha-alias\s+([^:]+):\s*([^~]+)~").ok();

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
    // look for ~kosha-dir {alias}: {folder}~ pattern
    let dir_pattern = Regex::new(r"~kosha-dir\s+([^:]+):\s*([^~]+)~").ok();

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
