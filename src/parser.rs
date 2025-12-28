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
}

/// A bhasya is a mula mantra (मूल मंत्र) with its commentary (the complete teaching unit)
/// Mula mantra uses **^mantra^** syntax inside quote blocks
/// - `> **^mantra^**` - creates a bhasya
/// - `>> **^mantra^**` - deprecates an existing bhasya
/// - `shastra: name\n> **^mantra^**` - quotes a bhasya from another shastra
#[derive(Debug, Clone)]
pub struct Bhasya {
    pub mantra_text: String,
    pub commentary: String,
    pub file: String,
    pub line: usize,
    /// True if this bhasya is deprecated (uses >> instead of >)
    pub is_deprecated: bool,
    /// If set, this bhasya is quoting from another shastra
    pub shastra: Option<String>,
}

/// An anusrit (अनुसृत) is a mantra reference using _| mantra |_ syntax
#[derive(Debug, Clone)]
pub struct Anusrit {
    pub mantra_text: String,
    pub file: String,
    pub line: usize,
    /// `_| mantra |_@shastra-name` for external anusrits
    pub shastra: Option<String>,
}

/// Shastra configuration loaded from .vyasa/shastra.json
/// Maps alias names to paths (local folders)
#[derive(Debug, Default)]
pub struct ShastraConfig {
    /// alias -> path mapping (merged from shastra.json and shastra.local.json)
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct Repository {
    pub mantras: HashMap<String, Mantra>,
    pub bhasyas: Vec<Bhasya>,
    pub anusrits: Vec<Anusrit>,
    pub shastra_config: ShastraConfig,
}

impl Repository {
    // _| vyasa check checks all non human meant files |_
    pub fn parse(path: &Path) -> Result<Self, String> {
        let mut repo = Repository::default();

        // find repository root and load shastra config
        let repo_root = find_repo_root(path);
        if let Some(root) = &repo_root {
            repo.shastra_config = load_shastra_config(root);
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

        // determine which mantras have explanations (commentary in same paragraph)
        mark_explained_mantras(&mut repo);

        Ok(repo)
    }

    pub fn unreferenced_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| !self.anusrits.iter().any(|r| r.mantra_text == m.text))
            .collect()
    }

    pub fn unexplained_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| !m.has_explanation)
            .collect()
    }

    pub fn anusrit_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for anusrit in &self.anusrits {
            *counts.entry(anusrit.mantra_text.clone()).or_insert(0) += 1;
        }
        counts
    }
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
    /// True if this paragraph uses >> (deprecation) instead of >
    is_deprecated: bool,
    /// If set, this paragraph is attributed to a shastra (from preceding `shastra: name` line)
    shastra: Option<String>,
}

fn extract_paragraphs(lines: &[&str], skip_lines: &[bool]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current_lines: Vec<(usize, String)> = Vec::new();
    let mut start_line = 0;
    let mut in_quote_block = false;
    let mut is_deprecated = false;
    let mut current_shastra: Option<String> = None;
    let mut pending_shastra: Option<String> = None;

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
                    is_deprecated,
                    shastra: current_shastra.take(),
                });
                in_quote_block = false;
                is_deprecated = false;
            }
            pending_shastra = None;
            continue;
        }

        let trimmed = line.trim_start();
        // check for >> (deprecation) vs > (normal quote)
        let is_deprecated_line = trimmed.starts_with(">>");
        let is_quote_line = trimmed.starts_with('>');
        let is_empty = line.trim().is_empty();

        // check for `shastra: name` pattern (must be alone on its line)
        if !in_quote_block && !is_quote_line && !is_empty {
            if let Some(shastra_name) = trimmed.strip_prefix("shastra:") {
                let shastra_name = shastra_name.trim();
                if !shastra_name.is_empty() {
                    // this is a shastra attribution line - remember it for next quote block
                    pending_shastra = Some(shastra_name.to_string());
                    continue; // don't include this line in any paragraph
                }
            }
        }

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
                    is_deprecated,
                    shastra: current_shastra.take(),
                });
                in_quote_block = false;
                is_deprecated = false;
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
                        is_deprecated: false,
                        shastra: None,
                    });
                }
                // empty line also clears pending shastra
                pending_shastra = None;
            } else if is_quote_line {
                // starting a quote block
                if !current_lines.is_empty() {
                    // end previous non-quote paragraph
                    let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                    paragraphs.push(Paragraph {
                        text,
                        start_line,
                        lines: std::mem::take(&mut current_lines),
                        is_deprecated: false,
                        shastra: None,
                    });
                }
                in_quote_block = true;
                is_deprecated = is_deprecated_line;
                current_shastra = pending_shastra.take();
                start_line = line_num;
                current_lines.push((line_num, line.to_string()));
            } else {
                // regular line
                if current_lines.is_empty() {
                    start_line = line_num;
                }
                current_lines.push((line_num, line.to_string()));
                // regular line clears pending shastra
                pending_shastra = None;
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
            is_deprecated,
            shastra: current_shastra,
        });
    }

    paragraphs
}

fn parse_paragraph(para: &Paragraph, file_name: &str, repo: &mut Repository) {
    // check if this is a quote block (bhasyas require quote blocks)
    let is_quote_block = para.text.trim_start().starts_with('>');

    if is_quote_block {
        // strip > or >> prefix from paragraph text for commentary extraction
        let unquoted_para: String = para.text
            .lines()
            .map(|l| {
                let trimmed = l.trim_start();
                // strip >> or > prefix
                let without_prefix = trimmed.strip_prefix(">>")
                    .unwrap_or_else(|| trimmed.strip_prefix('>').unwrap_or(trimmed));
                without_prefix.trim_start()
            })
            .collect::<Vec<_>>()
            .join("\n");

        // parse each line (strip > or >> prefix) for mantras and references
        for (line_num, line) in &para.lines {
            let trimmed = line.trim_start();
            let unquoted_line = trimmed.strip_prefix(">>")
                .unwrap_or_else(|| trimmed.strip_prefix('>').unwrap_or(trimmed))
                .trim_start();
            parse_line_with_paragraph(
                unquoted_line,
                file_name,
                *line_num,
                &unquoted_para,
                repo,
                true,
                para.is_deprecated,
                para.shastra.clone(),
            );
        }
    } else {
        // not a quote block - only parse for references, not bhasyas
        for (line_num, line) in &para.lines {
            parse_line_with_paragraph(line, file_name, *line_num, &para.text, repo, false, false, None);
        }
    }
}

fn parse_line_with_paragraph(
    line: &str,
    file_name: &str,
    line_num: usize,
    paragraph: &str,
    repo: &mut Repository,
    allow_bhasyas: bool,
    is_deprecated: bool,
    shastra: Option<String>,
) {
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
                    let commentary = extract_paragraph_commentary(paragraph, &mantra_text);
                    let has_explanation = !commentary.is_empty();

                    repo.bhasyas.push(Bhasya {
                        mantra_text: mantra_text.clone(),
                        commentary,
                        file: file_name.to_string(),
                        line: line_num,
                        is_deprecated,
                        shastra: shastra.clone(),
                    });

                    // only add to mantras if this is not a quoted/external bhasya
                    if shastra.is_none() {
                        repo.mantras.entry(mantra_text.clone()).or_insert(Mantra {
                            text: mantra_text,
                            file: file_name.to_string(),
                            line: line_num,
                            has_explanation,
                        });
                    }
                }
                continue;
            }
        }

        // reference with optional @shastra
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
                // check for @shastra suffix
                let mut shastra_ref = None;
                if chars.peek() == Some(&'@') {
                    chars.next(); // consume @
                    let mut shastra_name = String::new();
                    for c in chars.by_ref() {
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            shastra_name.push(c);
                        } else {
                            break;
                        }
                    }
                    if !shastra_name.is_empty() {
                        shastra_ref = Some(shastra_name);
                    }
                }

                repo.anusrits.push(Anusrit {
                    mantra_text: ref_text,
                    file: file_name.to_string(),
                    line: line_num,
                    shastra: shastra_ref,
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

fn load_shastra_config(repo_root: &Path) -> ShastraConfig {
    let mut config = ShastraConfig::default();

    // load .vyasa/shastra.json - main shastra aliases
    let shastra_file = repo_root.join(".vyasa/shastra.json");
    if let Ok(content) = fs::read_to_string(&shastra_file) {
        if let Ok(aliases) = serde_json::from_str::<HashMap<String, String>>(&content) {
            config.aliases = aliases;
        }
    }

    // load .vyasa/shastra.local.json - local overrides (gitignored)
    let local_file = repo_root.join(".vyasa/shastra.local.json");
    if let Ok(content) = fs::read_to_string(&local_file) {
        if let Ok(local_aliases) = serde_json::from_str::<HashMap<String, String>>(&content) {
            // local overrides main config
            for (alias, path) in local_aliases {
                config.aliases.insert(alias, path);
            }
        }
    }

    config
}
