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
/// - `>> **^mantra^**` - deprecates an existing bhasya (tyakta)
/// - `shastra: name\n> **^mantra^**` - quotes a bhasya from another shastra (uddhrit)
/// - `khandita: name\n> **^mantra^**` - refutes a bhasya from another shastra
#[derive(Debug, Clone)]
pub struct Bhasya {
    pub mantra_text: String,
    pub commentary: String,
    pub file: String,
    pub line: usize,
    /// True if this bhasya is deprecated (uses >> instead of >)
    pub is_deprecated: bool,
    /// If set, this bhasya is quoting from another shastra (uddhrit)
    pub shastra: Option<String>,
    /// If set, this bhasya is refuting a bhasya from another shastra (khandita)
    pub khandita: Option<String>,
}

/// An anusrit (अनुसृत) is a mantra reference using `_| mantra text |_` syntax
#[derive(Debug, Clone)]
pub struct Anusrit {
    pub mantra_text: String,
    pub file: String,
    pub line: usize,
    /// `_| mantra |_@shastra-name` for external anusrits
    pub shastra: Option<String>,
}

/// Repository configuration loaded from .vyasa/config.json
#[derive(Debug, Default)]
pub struct Config {
    /// Name of this shastra (used for self-references)
    pub name: Option<String>,
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
    pub config: Config,
    pub shastra_config: ShastraConfig,
}

impl Repository {
    // _| vyasa check checks all non human meant files |_
    pub fn parse(path: &Path) -> Result<Self, String> {
        let mut repo = Repository::default();

        // find repository root and load config
        let repo_root = find_repo_root(path);
        if let Some(root) = &repo_root {
            repo.config = load_config(root);
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

    pub fn unexplained_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| !m.has_explanation)
            .collect()
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
    lines: Vec<(usize, String)>,  // (line_num, text)
    /// True if this paragraph uses >> (deprecation) instead of >
    is_deprecated: bool,
    /// If set, this paragraph is attributed to a shastra (from preceding `shastra: name` line)
    shastra: Option<String>,
    /// If set, this paragraph refutes a bhasya from another shastra (from preceding `khandita: name` line)
    khandita: Option<String>,
}

/// Strip common comment prefixes from a line, returning the content after the prefix
/// Returns (stripped_content, comment_prefix) or None if no comment prefix found
fn strip_comment_prefix(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();

    // try each comment prefix in order of specificity
    let prefixes = ["//", "#", "--", ";", "%"];

    for prefix in prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            // calculate where the prefix starts in original line
            let prefix_start = line.len() - trimmed.len();
            let prefix_end = prefix_start + prefix.len();
            return Some((rest, &line[..prefix_end]));
        }
    }

    None
}

/// Check if a line is a comment-only line (just the comment prefix, maybe with whitespace)
fn is_comment_only_line(line: &str) -> bool {
    if let Some((rest, _)) = strip_comment_prefix(line) {
        rest.trim().is_empty()
    } else {
        false
    }
}

fn extract_paragraphs(lines: &[&str], skip_lines: &[bool]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current_lines: Vec<(usize, String)> = Vec::new();
    let mut in_quote_block = false;
    let mut is_deprecated = false;
    let mut current_shastra: Option<String> = None;
    let mut pending_shastra: Option<String> = None;
    let mut current_khandita: Option<String> = None;
    let mut pending_khandita: Option<String> = None;
    let mut current_comment_prefix: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if skip_lines[i] {
            // end current paragraph if any
            if !current_lines.is_empty() {
                let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                paragraphs.push(Paragraph {
                    text,
                    lines: std::mem::take(&mut current_lines),
                    is_deprecated,
                    shastra: current_shastra.take(),
                    khandita: current_khandita.take(),
                });
                in_quote_block = false;
                is_deprecated = false;
                current_comment_prefix = None;
            }
            pending_shastra = None;
            pending_khandita = None;
            continue;
        }

        // try to strip comment prefix for source code files
        let (content, comment_prefix) = if let Some((rest, prefix)) = strip_comment_prefix(line) {
            (rest.trim_start(), Some(prefix))
        } else {
            (line.trim_start(), None)
        };

        // check for >> (deprecation) vs > (normal quote)
        let is_deprecated_line = content.starts_with(">>");
        let is_quote_line = content.starts_with('>');
        let is_empty = line.trim().is_empty() || is_comment_only_line(line);

        // check for `shastra: name` or `khandita: name` pattern (must be alone on its line)
        if !in_quote_block && !is_quote_line && !is_empty {
            if let Some(shastra_name) = content.strip_prefix("shastra:") {
                let shastra_name = shastra_name.trim();
                if !shastra_name.is_empty() {
                    // this is a shastra attribution line - remember it for next quote block
                    pending_shastra = Some(shastra_name.to_string());
                    pending_khandita = None; // shastra and khandita are mutually exclusive
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    continue; // don't include this line in any paragraph
                }
            }
            if let Some(khandita_name) = content.strip_prefix("khandita:") {
                let khandita_name = khandita_name.trim();
                if !khandita_name.is_empty() {
                    // this is a khandita (refutation) line - remember it for next quote block
                    pending_khandita = Some(khandita_name.to_string());
                    pending_shastra = None; // shastra and khandita are mutually exclusive
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    continue; // don't include this line in any paragraph
                }
            }
        }

        if in_quote_block {
            // inside a quote block - continue until non-quote, non-empty line
            // for comment blocks, also check the comment prefix matches
            let same_comment_style = match (&current_comment_prefix, comment_prefix) {
                (Some(current), Some(prefix)) => current == prefix,
                (None, None) => true,
                _ => false,
            };

            // check if deprecation status changed (>> vs >) - if so, end current block and start new
            let deprecation_changed = is_quote_line && (is_deprecated != is_deprecated_line);

            if is_quote_line && same_comment_style && !deprecation_changed {
                current_lines.push((line_num, line.to_string()));
            } else if is_empty && same_comment_style {
                // empty line inside quote block - include it to preserve structure
                if let Some(ref prefix) = current_comment_prefix {
                    current_lines.push((line_num, format!("{} >", prefix)));
                } else {
                    current_lines.push((line_num, ">".to_string()));
                }
            } else {
                // non-quote line, different comment style, or deprecation changed - ends the quote block
                let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                paragraphs.push(Paragraph {
                    text,
                    lines: std::mem::take(&mut current_lines),
                    is_deprecated,
                    shastra: current_shastra.take(),
                    khandita: current_khandita.take(),
                });

                // if this line is also a quote line (e.g., deprecation changed), start a new quote block
                if is_quote_line && same_comment_style {
                    in_quote_block = true;
                    is_deprecated = is_deprecated_line;
                    current_shastra = pending_shastra.take();
                    current_khandita = pending_khandita.take();
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    current_lines.push((line_num, line.to_string()));
                } else {
                    in_quote_block = false;
                    is_deprecated = false;
                    current_comment_prefix = None;
                    // start new paragraph with this line if not empty
                    if !is_empty {
                        current_lines.push((line_num, line.to_string()));
                    }
                }
            }
        } else {
            // not in quote block
            if is_empty {
                // empty line ends paragraph
                if !current_lines.is_empty() {
                    let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                    paragraphs.push(Paragraph {
                        text,
                        lines: std::mem::take(&mut current_lines),
                        is_deprecated: false,
                        shastra: None,
                        khandita: None,
                    });
                }
                // empty line also clears pending shastra/khandita
                pending_shastra = None;
                pending_khandita = None;
                current_comment_prefix = None;
            } else if is_quote_line {
                // starting a quote block
                if !current_lines.is_empty() {
                    // end previous non-quote paragraph
                    let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
                    paragraphs.push(Paragraph {
                        text,
                        lines: std::mem::take(&mut current_lines),
                        is_deprecated: false,
                        shastra: None,
                        khandita: None,
                    });
                }
                in_quote_block = true;
                is_deprecated = is_deprecated_line;
                current_shastra = pending_shastra.take();
                current_khandita = pending_khandita.take();
                current_comment_prefix = comment_prefix.map(|s| s.to_string());
                current_lines.push((line_num, line.to_string()));
            } else {
                // regular line
                current_lines.push((line_num, line.to_string()));
                // regular line clears pending shastra/khandita
                pending_shastra = None;
                pending_khandita = None;
                current_comment_prefix = None;
            }
        }
    }

    // don't forget the last paragraph
    if !current_lines.is_empty() {
        let text = current_lines.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
        paragraphs.push(Paragraph {
            text,
            lines: current_lines,
            is_deprecated,
            shastra: current_shastra,
            khandita: current_khandita,
        });
    }

    paragraphs
}

/// Strip comment prefix and quote markers from a line
fn strip_comment_and_quote(line: &str) -> &str {
    // first strip comment prefix if present
    let after_comment = if let Some((rest, _)) = strip_comment_prefix(line) {
        rest.trim_start()
    } else {
        line.trim_start()
    };

    // then strip >> or > quote prefix
    after_comment.strip_prefix(">>")
        .unwrap_or_else(|| after_comment.strip_prefix('>').unwrap_or(after_comment))
        .trim_start()
}

/// Check if paragraph text starts with a quote marker (after stripping comment prefix)
fn is_quote_paragraph(text: &str) -> bool {
    if let Some(first_line) = text.lines().next() {
        let content = if let Some((rest, _)) = strip_comment_prefix(first_line) {
            rest.trim_start()
        } else {
            first_line.trim_start()
        };
        content.starts_with('>')
    } else {
        false
    }
}

fn parse_paragraph(para: &Paragraph, file_name: &str, repo: &mut Repository) {
    // check if this is a quote block (bhasyas require quote blocks)
    let is_quote_block = is_quote_paragraph(&para.text);

    if is_quote_block {
        // strip comment prefix and > or >> from paragraph text for commentary extraction
        let unquoted_para: String = para.text
            .lines()
            .map(strip_comment_and_quote)
            .collect::<Vec<_>>()
            .join("\n");

        // parse each line (strip comment prefix and > or >> prefix) for mantras and references
        for (line_num, line) in &para.lines {
            let unquoted_line = strip_comment_and_quote(line);
            parse_line_with_paragraph(
                unquoted_line,
                file_name,
                *line_num,
                &unquoted_para,
                repo,
                true,
                para.is_deprecated,
                para.shastra.clone(),
                para.khandita.clone(),
            );
        }
    } else {
        // not a quote block - only parse for references, not bhasyas
        for (line_num, line) in &para.lines {
            // still strip comment prefix for anusrit detection in code
            let content = if let Some((rest, _)) = strip_comment_prefix(line) {
                rest
            } else {
                line.as_str()
            };
            parse_line_with_paragraph(content, file_name, *line_num, &para.text, repo, false, false, None, None);
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
    khandita: Option<String>,
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
                        khandita: khandita.clone(),
                    });

                    // only add to mantras if this is a mula bhasya (not quoted, not tyakta, not khandita)
                    if shastra.is_none() && khandita.is_none() && !is_deprecated {
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

fn load_config(repo_root: &Path) -> Config {
    let config_file = repo_root.join(".vyasa/config.json");
    if let Ok(content) = fs::read_to_string(&config_file) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            return Config {
                name: json.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };
        }
    }
    Config::default()
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
