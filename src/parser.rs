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

/// A bhasya is a mantra with its commentary (the complete teaching unit)
#[derive(Debug, Clone)]
pub struct Bhasya {
    pub mantra_text: String,
    pub commentary: String,
    pub file: String,
    pub line: usize,
    pub is_template: bool,
}

// `^mantra^@kosha` provides commentary on external mantra
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
    // `_| mantra |_@kosha-name` for external references
    pub kosha: Option<String>,
}

// _| kosha-alias {kosha-alias}: {kosha-value} |_
#[derive(Debug, Clone)]
pub struct KoshaAlias {
    pub alias: String,
    pub value: String,
}

// _| kosha-dir {kosha-alias}: {folder-name} |_
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
    pub bhasyas: Vec<Bhasya>,
    pub references: Vec<Reference>,
    pub external_commentaries: Vec<ExternalCommentary>,
    pub kosha_config: KoshaConfig,
}

impl Repository {
    // _| vyasa check checks all non human meant files |_
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

            // _| vyasa check checks all non human meant files |_
            // skip binary and human-meant files (xml, images, etc.)
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if should_skip_file(ext) {
                continue;
            }

            // skip canon.md - it's a digest file, not a source
            if file_path.file_name().map_or(false, |n| n == "canon.md") {
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

// _| mantras should use inline syntax not block because they are meant to be short |_
fn parse_file(content: &str, file_name: &str, repo: &mut Repository) {
    let lines: Vec<&str> = content.lines().collect();

    // first pass: identify code block regions to skip
    let mut in_code_block = false;
    let mut skip_lines: Vec<bool> = vec![false; lines.len()];

    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            skip_lines[i] = true;
        } else {
            skip_lines[i] = in_code_block;
        }
    }

    // second pass: identify paragraphs and parse them
    let paragraphs = extract_paragraphs(&lines, &skip_lines);

    for para in paragraphs {
        parse_paragraph(&para, file_name, repo);
    }
}

#[derive(Debug)]
struct Paragraph {
    text: String,
    start_line: usize,  // 1-indexed
    lines: Vec<(usize, String)>,  // (line_num, text)
}

fn extract_paragraphs(lines: &[&str], skip_lines: &[bool]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current_lines: Vec<(usize, String)> = Vec::new();
    let mut start_line = 0;
    let mut in_quote_block = false;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if skip_lines[i] {
            // end current paragraph if any
            if !current_lines.is_empty() {
                let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                paragraphs.push(Paragraph {
                    text,
                    start_line,
                    lines: std::mem::take(&mut current_lines),
                });
                in_quote_block = false;
            }
            continue;
        }

        let is_quote_line = line.trim_start().starts_with('>');
        let is_empty = line.trim().is_empty();

        if in_quote_block {
            // inside a quote block - continue until non-quote, non-empty line
            if is_quote_line {
                current_lines.push((line_num, line.to_string()));
            } else if is_empty {
                // empty line inside quote block - include it to preserve structure
                current_lines.push((line_num, ">".to_string()));
            } else {
                // non-quote line ends the quote block
                let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                paragraphs.push(Paragraph {
                    text,
                    start_line,
                    lines: std::mem::take(&mut current_lines),
                });
                in_quote_block = false;
                // start new paragraph with this line
                start_line = line_num;
                current_lines.push((line_num, line.to_string()));
            }
        } else {
            // not in quote block
            if is_empty {
                // empty line ends paragraph
                if !current_lines.is_empty() {
                    let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                    paragraphs.push(Paragraph {
                        text,
                        start_line,
                        lines: std::mem::take(&mut current_lines),
                    });
                }
            } else if is_quote_line {
                // starting a quote block
                if !current_lines.is_empty() {
                    // end previous non-quote paragraph
                    let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                    paragraphs.push(Paragraph {
                        text,
                        start_line,
                        lines: std::mem::take(&mut current_lines),
                    });
                }
                in_quote_block = true;
                start_line = line_num;
                current_lines.push((line_num, line.to_string()));
            } else {
                // regular line
                if current_lines.is_empty() {
                    start_line = line_num;
                }
                current_lines.push((line_num, line.to_string()));
            }
        }
    }

    // don't forget the last paragraph
    if !current_lines.is_empty() {
        let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
        paragraphs.push(Paragraph {
            text,
            start_line,
            lines: current_lines,
        });
    }

    paragraphs
}

fn parse_paragraph(para: &Paragraph, file_name: &str, repo: &mut Repository) {
    // check if this is a quote block (bhasyas require quote blocks)
    let is_quote_block = para.text.trim_start().starts_with('>');

    if is_quote_block {
        // strip > prefix from paragraph text for commentary extraction
        let unquoted_para: String = para.text
            .lines()
            .map(|l| l.trim_start().strip_prefix('>').unwrap_or(l).trim_start())
            .collect::<Vec<_>>()
            .join("\n");

        // parse each line (strip > prefix) for mantras and references
        for (line_num, line) in &para.lines {
            let unquoted_line = line.trim_start()
                .strip_prefix('>')
                .unwrap_or(line)
                .trim_start();
            parse_line_with_paragraph(unquoted_line, file_name, *line_num, &unquoted_para, repo, true);
        }
    } else {
        // not a quote block - only parse for references, not bhasyas
        for (line_num, line) in &para.lines {
            parse_line_with_paragraph(line, file_name, *line_num, &para.text, repo, false);
        }
    }
}

fn parse_line_with_paragraph(line: &str, file_name: &str, line_num: usize, paragraph: &str, repo: &mut Repository, allow_bhasyas: bool) {
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

        // **^mantra^** - bhasya syntax (mantra with commentary)
        // or **^mantra^**@kosha for external commentary
        // only allowed inside quote blocks (> ...)
        if allow_bhasyas && c == '*' && chars.peek() == Some(&'*') {
            chars.next(); // consume second *
            if chars.peek() == Some(&'^') {
                chars.next(); // consume ^
                let mut mantra_text = String::new();

                for c in chars.by_ref() {
                    if c == '^' {
                        break;
                    }
                    mantra_text.push(c);
                }

                // consume closing **
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                    }
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
                        let commentary = extract_paragraph_commentary(paragraph, &mantra_text);
                        let has_explanation = !commentary.is_empty();

                        repo.bhasyas.push(Bhasya {
                            mantra_text: mantra_text.clone(),
                            commentary,
                            file: file_name.to_string(),
                            line: line_num,
                            is_template,
                        });

                        repo.mantras.entry(mantra_text.clone()).or_insert(Mantra {
                            text: mantra_text,
                            file: file_name.to_string(),
                            line: line_num,
                            has_explanation,
                            is_template,
                        });
                    }
                }
                continue;
            }
        }

        // reference with optional @kosha
        if c == '_' && chars.peek() == Some(&'|') {
            chars.next(); // consume |
            let mut ref_text = String::new();

            loop {
                match chars.next() {
                    Some('|') if chars.peek() == Some(&'_') => {
                        chars.next(); // consume _
                        break;
                    }
                    Some(c) => ref_text.push(c),
                    None => break,
                }
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


// Extract commentary from paragraph (entire paragraph minus mantras)
fn extract_paragraph_commentary(paragraph: &str, mantra_text: &str) -> String {
    // remove all mantra markers from the paragraph
    let mut result = paragraph.to_string();

    // remove this specific mantra definition (**^mantra^** syntax only)
    let bold_pattern = format!("**^{}^**", mantra_text);
    result = result.replace(&bold_pattern, "");

    // also remove any other mantra markers in the paragraph
    // (a paragraph might define multiple mantras)
    let mut cleaned = String::new();
    let mut chars = result.chars().peekable();
    let mut in_mantra = false;

    while let Some(c) = chars.next() {
        // handle **^ bold mantra start
        if c == '*' && chars.peek() == Some(&'*') {
            let mut peek_chars = chars.clone();
            peek_chars.next(); // skip second *
            if peek_chars.peek() == Some(&'^') {
                chars.next(); // consume second *
                chars.next(); // consume ^
                in_mantra = true;
                continue;
            }
        }
        // handle ^** bold mantra end
        if c == '^' && in_mantra {
            // check for ** after
            if chars.peek() == Some(&'*') {
                let mut peek_chars = chars.clone();
                peek_chars.next();
                if peek_chars.peek() == Some(&'*') {
                    chars.next(); // consume first *
                    chars.next(); // consume second *
                }
            }
            in_mantra = false;
            continue;
        }
        if !in_mantra {
            cleaned.push(c);
        }
    }

    // clean up the result
    let trimmed = cleaned
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // strip leading " - " which is common commentary prefix
    let trimmed = trimmed.trim();
    let trimmed = trimmed.strip_prefix("- ").unwrap_or(trimmed);

    trimmed.to_string()
}

// _| mantra commentary can be in same para |_ - mark mantras as explained if they have nearby commentary
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

// _| template placeholders can include example values as {name=example} |_
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

// _| vyasa check checks all non human meant files |_
// human-meant: configs, data files, binaries - skip these
// source code and docs: scan for mantras
fn should_skip_file(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(),
        // binary/compiled
        "exe" | "dll" | "so" | "dylib" | "o" | "a" | "class" | "pyc" | "pyo" |
        // images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "svg" | "webp" |
        // audio/video
        "mp3" | "mp4" | "wav" | "avi" | "mov" | "mkv" |
        // archives
        "zip" | "tar" | "gz" | "bz2" | "7z" | "rar" |
        // human-meant data/config (often auto-generated or verbose)
        "xml" | "plist" | "pbxproj" |
        // fonts
        "ttf" | "otf" | "woff" | "woff2" |
        // documents (human-readable but not code)
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" |
        // lock files
        "lock"
    )
}

pub fn find_repo_root(path: &Path) -> Option<std::path::PathBuf> {
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

    // _| .vyasa/kosha.md contains kosha configuration |_
    let kosha_file = repo_root.join(".vyasa/kosha.md");
    if let Ok(content) = fs::read_to_string(&kosha_file) {
        parse_kosha_aliases(&content, &mut config);
    }

    // _| .vyasa/kosha.local.md stores local folder overrides |_
    let local_file = repo_root.join(".vyasa/kosha.local.md");
    if let Ok(content) = fs::read_to_string(&local_file) {
        parse_kosha_local(&content, &mut config);
    }

    config
}

fn parse_kosha_aliases(content: &str, config: &mut KoshaConfig) {
    // look for _kosha-alias {alias}: {value}_ pattern
    let alias_pattern = Regex::new(r"_kosha-alias\s+([^:]+):\s*([^_]+)_").ok();

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
    // look for _kosha-dir {alias}: {folder}_ pattern
    let dir_pattern = Regex::new(r"_kosha-dir\s+([^:]+):\s*([^_]+)_").ok();

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
