use crate::parser::{Bhasya, Repository};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// The canon.md file contains accepted mantras for this kosha
/// Supports versioning: canon.md (single) or 001.md, 002.md, etc. (versioned)
#[derive(Debug, Clone, Default)]
pub struct Canon {
    /// Mantra text -> canonical commentary
    pub entries: HashMap<String, CanonEntry>,
    /// Path to the canon file (if found)
    pub path: Option<String>,
    /// Version number (None for canon.md, Some(n) for numbered files)
    pub version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CanonEntry {
    pub mantra: String,
    pub commentary: String,
    pub source_file: String,
    /// If this mantra is from another kosha (^mantra^@kosha)
    pub external_kosha: Option<String>,
}

/// Result of searching for canon.md
pub enum CanonSearchResult {
    /// No canon.md found
    NotFound,
    /// Found exactly one canon.md
    Found(Canon),
    /// Found multiple canon.md files (error)
    Multiple(Vec<String>),
    /// Canon file has invalid content
    Invalid { path: String, errors: Vec<String> },
}

impl Canon {
    /// Search for canon file at repo root
    /// Supports: canon.md (single) or versioned files like 001.md, 002.md, etc.
    /// When versioned, only the highest numbered file is used for checking.
    pub fn find(repo_root: &Path) -> CanonSearchResult {
        // First, look for versioned canon files (numbered .md files)
        let mut versioned_files: Vec<(u32, String)> = Vec::new();

        if let Ok(entries) = fs::read_dir(repo_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if filename is a number followed by .md
                        if let Some(stem) = name.strip_suffix(".md") {
                            if let Ok(version) = stem.parse::<u32>() {
                                versioned_files.push((version, path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }

        // If we have versioned files, use the highest version
        if !versioned_files.is_empty() {
            versioned_files.sort_by(|a, b| b.0.cmp(&a.0)); // Sort descending
            let (version, path) = &versioned_files[0];
            return Self::load_canon_file(path, Some(*version));
        }

        // Fall back to canon.md
        let canon_path = repo_root.join("canon.md");

        // Check for multiple canon.md files in subdirectories (error case)
        let mut found_paths = Vec::new();
        if canon_path.exists() {
            found_paths.push(canon_path.to_string_lossy().to_string());
        }

        // Also check in subdirectories for duplicates
        if let Ok(entries) = fs::read_dir(repo_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !path.file_name().map_or(false, |n| n.to_string_lossy().starts_with('.')) {
                    let subdir_canon = path.join("canon.md");
                    if subdir_canon.exists() {
                        found_paths.push(subdir_canon.to_string_lossy().to_string());
                    }
                }
            }
        }

        if found_paths.is_empty() {
            return CanonSearchResult::NotFound;
        }

        if found_paths.len() > 1 {
            return CanonSearchResult::Multiple(found_paths);
        }

        Self::load_canon_file(&found_paths[0], None)
    }

    /// Load and validate a canon file
    fn load_canon_file(path: &str, version: Option<u32>) -> CanonSearchResult {
        match fs::read_to_string(path) {
            Ok(content) => {
                let errors = validate_canon_content(&content);
                if !errors.is_empty() {
                    return CanonSearchResult::Invalid {
                        path: path.to_string(),
                        errors,
                    };
                }
                let mut canon = Self::parse(&content);
                canon.path = Some(path.to_string());
                canon.version = version;
                CanonSearchResult::Found(canon)
            }
            Err(e) => CanonSearchResult::Invalid {
                path: path.to_string(),
                errors: vec![format!("failed to read file: {}", e)],
            },
        }
    }

    fn parse(content: &str) -> Self {
        let mut entries = HashMap::new();

        // parse canon.md with new format:
        // filename.md
        //
        // > ^mantra^ - commentary
        let lines: Vec<&str> = content.lines().collect();
        let paragraphs = extract_paragraphs(&lines);

        let mut current_file: Option<String> = None;

        for para in paragraphs {
            let text = para.text.trim();

            // skip headings
            if text.starts_with('#') {
                continue;
            }

            // check if this is a quote block (mantra/commentary)
            if text.starts_with('>') {
                let source_file = current_file.clone().unwrap_or_default();
                // remove > prefix from each line
                let unquoted: String = para
                    .text
                    .lines()
                    .map(|l| l.trim_start().strip_prefix('>').unwrap_or(l).trim_start())
                    .collect::<Vec<_>>()
                    .join("\n");

                // find mantras with optional @kosha suffix
                for (mantra, kosha) in extract_mantras_with_kosha(&unquoted) {
                    let commentary = extract_canon_commentary(&unquoted, &mantra);
                    entries.insert(
                        mantra.clone(),
                        CanonEntry {
                            mantra,
                            commentary,
                            source_file: source_file.clone(),
                            external_kosha: kosha,
                        },
                    );
                }
            } else {
                // this is a filename
                current_file = Some(text.to_string());
            }
        }

        Self {
            entries,
            path: None,
            version: None,
        }
    }

    pub fn get(&self, mantra: &str) -> Option<&CanonEntry> {
        self.entries.get(mantra)
    }

    /// Get all external entries (mantras from other koshas)
    pub fn external_entries(&self) -> Vec<&CanonEntry> {
        self.entries
            .values()
            .filter(|e| e.external_kosha.is_some())
            .collect()
    }
}

/// Validate canon.md content
/// Format: headings, filenames (plain text), and quote blocks with mantras
fn validate_canon_content(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let paragraphs = extract_paragraphs(&lines);

    let mut expecting_quote = false;
    let mut last_filename: Option<String> = None;

    for (i, para) in paragraphs.iter().enumerate() {
        let text = para.text.trim();

        // empty paragraphs are fine
        if text.is_empty() {
            continue;
        }

        // headings are allowed
        if text.starts_with('#') {
            let heading_part = text.trim_start_matches('#');
            if !heading_part.is_empty() && !heading_part.starts_with(' ') {
                errors.push(format!(
                    "paragraph {}: heading must have space after #: \"{}\"",
                    i + 1,
                    truncate(text, 40)
                ));
            }
            expecting_quote = false;
            continue;
        }

        // check if this is a quote block
        if text.starts_with('>') {
            // remove > prefix and check for mantras
            let unquoted: String = para
                .text
                .lines()
                .map(|l| l.trim_start().strip_prefix('>').unwrap_or(l).trim_start())
                .collect::<Vec<_>>()
                .join("\n");

            let mantras = extract_mantras(&unquoted);
            if mantras.is_empty() {
                errors.push(format!(
                    "paragraph {}: quote block must contain a mantra definition (**^mantra^**): \"{}\"",
                    i + 1,
                    truncate(text, 40)
                ));
            }
            if last_filename.is_none() {
                errors.push(format!(
                    "paragraph {}: quote block must be preceded by a filename",
                    i + 1
                ));
            }
            expecting_quote = false;
            last_filename = None;
        } else {
            // this should be a filename
            if expecting_quote {
                errors.push(format!(
                    "paragraph {}: expected a quote block (> ...) after filename, got: \"{}\"",
                    i + 1,
                    truncate(text, 40)
                ));
            }
            // validate it looks like a filename (has extension or is a path)
            if !text.contains('.') && !text.contains('/') {
                errors.push(format!(
                    "paragraph {}: expected a filename (e.g., docs/file.md): \"{}\"",
                    i + 1,
                    truncate(text, 40)
                ));
            }
            expecting_quote = true;
            last_filename = Some(text.to_string());
        }
    }

    if expecting_quote {
        if let Some(filename) = last_filename {
            errors.push(format!(
                "missing quote block after filename: {}",
                filename
            ));
        }
    }

    errors
}

// Simple paragraph extraction for canon.md parsing
struct Paragraph {
    text: String,
}

fn extract_paragraphs(lines: &[&str]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            if !current_lines.is_empty() {
                let text = current_lines.join("\n");
                paragraphs.push(Paragraph { text });
                current_lines.clear();
            }
        } else {
            current_lines.push(line);
        }
    }

    if !current_lines.is_empty() {
        let text = current_lines.join("\n");
        paragraphs.push(Paragraph { text });
    }

    paragraphs
}

/// Extract mantras with optional @kosha suffix
/// Uses **^mantra^** bold syntax (required)
/// Returns (mantra_text, optional_kosha)
fn extract_mantras_with_kosha(paragraph: &str) -> Vec<(String, Option<String>)> {
    let mut mantras = Vec::new();
    let mut chars = paragraph.chars().peekable();
    let mut in_backtick = false;

    while let Some(c) = chars.next() {
        if c == '`' {
            in_backtick = !in_backtick;
            continue;
        }
        if in_backtick {
            continue;
        }

        // look for **^ sequence
        if c == '*' && chars.peek() == Some(&'*') {
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
                if mantra_text.is_empty() {
                    continue;
                }

                // check for @kosha suffix
                let kosha = if chars.peek() == Some(&'@') {
                    chars.next(); // consume @
                    let mut kosha_name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            kosha_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if kosha_name.is_empty() {
                        None
                    } else {
                        Some(kosha_name)
                    }
                } else {
                    None
                };

                mantras.push((mantra_text, kosha));
            }
        }
    }

    mantras
}

fn extract_mantras(paragraph: &str) -> Vec<String> {
    extract_mantras_with_kosha(paragraph)
        .into_iter()
        .map(|(m, _)| m)
        .collect()
}

fn extract_canon_commentary(paragraph: &str, mantra_text: &str) -> String {
    // remove all mantra markers from the paragraph (**^mantra^** syntax)
    let mut result = paragraph.to_string();
    let pattern = format!("**^{}^**", mantra_text);
    result = result.replace(&pattern, "");

    // remove any other mantra markers
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

    // preserve original line structure, just trim each line
    let lines: Vec<&str> = cleaned.lines().collect();
    let mut trimmed_lines: Vec<&str> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        // skip leading empty lines
        if trimmed.is_empty() && trimmed_lines.is_empty() {
            continue;
        }
        trimmed_lines.push(trimmed);
    }

    // remove trailing empty lines
    while trimmed_lines.last().map_or(false, |l| l.is_empty()) {
        trimmed_lines.pop();
    }

    let result = trimmed_lines.join("\n");

    // strip leading "- " if present
    result
        .strip_prefix("- ")
        .unwrap_or(&result)
        .to_string()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

/// Status of a mantra compared to canon
#[derive(Debug)]
pub enum MantraStatus {
    /// Mantra exists in canon with matching commentary
    Accepted,
    /// Mantra not in canon yet
    New,
    /// Mantra in canon but commentary differs
    Changed { canon_commentary: String },
    /// Mantra only exists in canon, not in any source file (error)
    OrphanedInCanon { canon_commentary: String },
}

/// A unique mantra with all its bhasyas from the repo
#[derive(Debug)]
pub struct MantraWithStatus {
    pub mantra_text: String,
    pub bhasyas: Vec<Bhasya>,
    pub status: MantraStatus,
}

/// Compare repo mantras against canon
pub fn compare_with_canon(repo: &Repository, canon: &Canon) -> Vec<MantraWithStatus> {
    // group bhasyas by mantra text
    let mut by_mantra: HashMap<String, Vec<Bhasya>> = HashMap::new();
    for bhasya in &repo.bhasyas {
        by_mantra
            .entry(bhasya.mantra_text.clone())
            .or_default()
            .push(bhasya.clone());
    }

    let mut results = Vec::new();

    // check mantras found in repo
    for (mantra_text, bhasyas) in &by_mantra {
        let status = if let Some(canon_entry) = canon.get(mantra_text) {
            // mantra is in canon - check if any bhasya matches
            let any_matches = bhasyas
                .iter()
                .any(|b| normalize_commentary(&b.commentary) == normalize_commentary(&canon_entry.commentary));

            if any_matches {
                MantraStatus::Accepted
            } else {
                MantraStatus::Changed {
                    canon_commentary: canon_entry.commentary.clone(),
                }
            }
        } else {
            MantraStatus::New
        };

        results.push(MantraWithStatus {
            mantra_text: mantra_text.clone(),
            bhasyas: bhasyas.clone(),
            status,
        });
    }

    // check for orphaned mantras (in canon but not in repo)
    for (mantra_text, canon_entry) in &canon.entries {
        if !by_mantra.contains_key(mantra_text) {
            results.push(MantraWithStatus {
                mantra_text: mantra_text.clone(),
                bhasyas: vec![],
                status: MantraStatus::OrphanedInCanon {
                    canon_commentary: canon_entry.commentary.clone(),
                },
            });
        }
    }

    // sort by mantra text
    results.sort_by(|a, b| a.mantra_text.cmp(&b.mantra_text));

    results
}

/// Normalize commentary for comparison (trim whitespace, collapse spaces)
fn normalize_commentary(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
