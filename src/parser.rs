use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Information about a mantra in this repository
#[derive(Debug, Clone, Default)]
pub struct MantraInfo {
    /// First definition location (for display)
    pub file: String,
    pub line: usize,
    /// Whether this mantra has commentary
    pub has_explanation: bool,
    /// Bhasya indices where this is a mula definition (**^mantra^**)
    pub mula_bhasyas: Vec<usize>,
    /// Bhasya indices where this is referenced inside bhasya (_| mantra |_)
    pub anusrit_bhasyas: Vec<usize>,
}

/// The kind of bhasya - determines its semantic meaning
#[derive(Debug, Clone, PartialEq)]
pub enum BhasyaKind {
    /// Regular bhasya - defines a mula mantra
    Mula,
    /// Quoting canonical mula-bhasya location (uddhrit उद्धृत) - same or other shastra
    Uddhrit(String),
    /// Refuting a shastra's bhasya (khandita खण्डित)
    Khandita(String),
    /// Deprecated bhasya (tyakta त्यक्त)
    Tyakta,
}

/// A bhasya is a quote block (the teaching unit containing mantras and commentary)
/// - `> **^mantra^**` - contains a mula mantra definition (Mula)
/// - `shastra: name\n> ...` - quotes canonical location in a shastra (Uddhrit)
/// - `khandita: name\n> ...` - refutes a shastra's bhasya (Khandita)
/// - `tyakta:\n> ...` - deprecates this bhasya (Tyakta)
#[derive(Debug, Clone)]
pub struct Bhasya {
    /// Original paragraph text (preserves structure with newlines)
    pub paragraph: String,
    pub file: String,
    pub line: usize,
    /// The kind of bhasya (mula, uddhrit, khandita, or tyakta)
    pub kind: BhasyaKind,
}

/// An anusrit (अनुसृत) is a mantra reference outside a bhasya using `_| mantra text |_` syntax
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
    /// All mantras indexed by text
    pub mantras: HashMap<String, MantraInfo>,
    /// All bhasyas (quote blocks)
    pub bhasyas: Vec<Bhasya>,
    /// Anusrits outside bhasyas (for validation)
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

    /// Get mantras without explanations
    pub fn unexplained_mantras(&self) -> Vec<(&str, &MantraInfo)> {
        self.mantras
            .iter()
            .filter(|(_, m)| !m.has_explanation)
            .map(|(text, info)| (text.as_str(), info))
            .collect()
    }

    /// Get all mula mantras with their associated bhasyas
    /// Returns (mantra_text, &Bhasya) for each mula occurrence
    pub fn mula_mantras_with_bhasyas(&self) -> Vec<(&str, &Bhasya)> {
        self.mantras
            .iter()
            .flat_map(|(mantra_text, info)| {
                info.mula_bhasyas.iter().filter_map(move |&idx| {
                    self.bhasyas.get(idx).map(|b| (mantra_text.as_str(), b))
                })
            })
            .collect()
    }

    /// Find all bhasyas (mula + anusrit) for a given mantra text
    pub fn bhasyas_for_mantra(&self, mantra_text: &str) -> Vec<&Bhasya> {
        self.mantras
            .get(mantra_text)
            .map(|info| {
                info.mula_bhasyas.iter()
                    .chain(info.anusrit_bhasyas.iter())
                    .filter_map(|&idx| self.bhasyas.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a mantra text exists as mula in any bhasya
    pub fn has_any_bhasya_for_mantra(&self, mantra_text: &str) -> bool {
        self.mantras
            .get(mantra_text)
            .map(|info| !info.mula_bhasyas.is_empty())
            .unwrap_or(false)
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
    /// True if this paragraph is deprecated (from preceding `tyakta:` line)
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
    let mut pending_tyakta = false;
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
            pending_tyakta = false;
            continue;
        }

        // try to strip comment prefix for source code files
        let (content, comment_prefix) = if let Some((rest, prefix)) = strip_comment_prefix(line) {
            (rest.trim_start(), Some(prefix))
        } else {
            (line.trim_start(), None)
        };

        // check for quote line (just > now, not >>)
        let is_quote_line = content.starts_with('>');
        let is_empty = line.trim().is_empty() || is_comment_only_line(line);

        // check for `shastra: name`, `khandita: name`, or `tyakta:` pattern (must be alone on its line)
        if !in_quote_block && !is_quote_line && !is_empty {
            if let Some(shastra_name) = content.strip_prefix("shastra:") {
                let shastra_name = shastra_name.trim();
                if !shastra_name.is_empty() {
                    // this is a shastra attribution line - remember it for next quote block
                    pending_shastra = Some(shastra_name.to_string());
                    pending_khandita = None; // these are mutually exclusive
                    pending_tyakta = false;
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    continue; // don't include this line in any paragraph
                }
            }
            if let Some(khandita_name) = content.strip_prefix("khandita:") {
                let khandita_name = khandita_name.trim();
                if !khandita_name.is_empty() {
                    // this is a khandita (refutation) line - remember it for next quote block
                    pending_khandita = Some(khandita_name.to_string());
                    pending_shastra = None; // these are mutually exclusive
                    pending_tyakta = false;
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    continue; // don't include this line in any paragraph
                }
            }
            if content.starts_with("tyakta:") {
                // this is a tyakta (deprecation) line - remember it for next quote block
                pending_tyakta = true;
                pending_shastra = None; // these are mutually exclusive
                pending_khandita = None;
                current_comment_prefix = comment_prefix.map(|s| s.to_string());
                continue; // don't include this line in any paragraph
            }
        }

        if in_quote_block {
            // inside a quote block - continue only if this is a quote line
            // for comment blocks, also check the comment prefix matches
            let same_comment_style = match (&current_comment_prefix, comment_prefix) {
                (Some(current), Some(prefix)) => current == prefix,
                (None, None) => true,
                _ => false,
            };

            if is_quote_line && same_comment_style {
                current_lines.push((line_num, line.to_string()));
            } else {
                // non-quote line (including empty lines) ends the quote block
                // in markdown, blank lines between > blocks create separate blocks
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

                // check for prefix lines that end the quote block
                if let Some(shastra_name) = content.strip_prefix("shastra:") {
                    let shastra_name = shastra_name.trim();
                    if !shastra_name.is_empty() {
                        pending_shastra = Some(shastra_name.to_string());
                        pending_khandita = None;
                        pending_tyakta = false;
                        current_comment_prefix = comment_prefix.map(|s| s.to_string());
                        continue;
                    }
                }
                if let Some(khandita_name) = content.strip_prefix("khandita:") {
                    let khandita_name = khandita_name.trim();
                    if !khandita_name.is_empty() {
                        pending_khandita = Some(khandita_name.to_string());
                        pending_shastra = None;
                        pending_tyakta = false;
                        current_comment_prefix = comment_prefix.map(|s| s.to_string());
                        continue;
                    }
                }
                if content.starts_with("tyakta:") {
                    pending_tyakta = true;
                    pending_shastra = None;
                    pending_khandita = None;
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    continue;
                }

                // if this line is a quote line, start a new quote block
                if is_quote_line {
                    in_quote_block = true;
                    is_deprecated = pending_tyakta;
                    pending_tyakta = false;
                    current_shastra = pending_shastra.take();
                    current_khandita = pending_khandita.take();
                    current_comment_prefix = comment_prefix.map(|s| s.to_string());
                    current_lines.push((line_num, line.to_string()));
                } else if !is_empty {
                    // start new paragraph with this line if not empty
                    current_lines.push((line_num, line.to_string()));
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
                // empty line also clears pending state
                pending_shastra = None;
                pending_khandita = None;
                pending_tyakta = false;
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
                is_deprecated = pending_tyakta;
                pending_tyakta = false;
                current_shastra = pending_shastra.take();
                current_khandita = pending_khandita.take();
                current_comment_prefix = comment_prefix.map(|s| s.to_string());
                current_lines.push((line_num, line.to_string()));
            } else {
                // regular line
                current_lines.push((line_num, line.to_string()));
                // regular line clears pending state
                pending_shastra = None;
                pending_khandita = None;
                pending_tyakta = false;
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

    // then strip > quote prefix
    after_comment.strip_prefix('>')
        .unwrap_or(after_comment)
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
        // strip comment prefix and > from paragraph text
        let unquoted_para: String = para.text
            .lines()
            .map(strip_comment_and_quote)
            .collect::<Vec<_>>()
            .join("\n");

        // first pass: check if this quote block contains any mula mantras
        let has_mula_mantra = para.lines.iter().any(|(_, line)| {
            let unquoted = strip_comment_and_quote(line);
            unquoted.starts_with("**^")
        });

        // only create a bhasya if it contains at least one mula mantra
        let bhasya_index = if has_mula_mantra {
            let start_line = para.lines.first().map(|(n, _)| *n).unwrap_or(0);
            let idx = repo.bhasyas.len();

            // determine the kind based on paragraph attributes
            let kind = if para.is_deprecated {
                BhasyaKind::Tyakta
            } else if let Some(ref shastra) = para.shastra {
                BhasyaKind::Uddhrit(shastra.clone())
            } else if let Some(ref khandita) = para.khandita {
                BhasyaKind::Khandita(khandita.clone())
            } else {
                BhasyaKind::Mula
            };

            repo.bhasyas.push(Bhasya {
                paragraph: unquoted_para.clone(),
                file: file_name.to_string(),
                line: start_line,
                kind,
            });
            Some(idx)
        } else {
            None
        };

        // second pass: parse each line for mantras and references
        for (line_num, line) in &para.lines {
            let unquoted_line = strip_comment_and_quote(line);
            parse_line_in_bhasya(
                unquoted_line,
                file_name,
                *line_num,
                repo,
                bhasya_index,
            );
        }
    } else {
        // not a quote block - only parse for anusrits, not mula mantras
        for (line_num, line) in &para.lines {
            let content = if let Some((rest, _)) = strip_comment_prefix(line) {
                rest
            } else {
                line.as_str()
            };
            parse_line_for_anusrits(content, file_name, *line_num, repo, None);
        }
    }
}

/// Parse a line inside a bhasya (quote block) for mula mantras and anusrits
fn parse_line_in_bhasya(
    line: &str,
    file_name: &str,
    line_num: usize,
    repo: &mut Repository,
    bhasya_index: Option<usize>,
) {
    let mut chars = line.chars().peekable();
    let mut in_backtick = false;
    let mut position = 0usize;

    while let Some(c) = chars.next() {
        // skip content inside backticks (inline code)
        if c == '`' {
            in_backtick = !in_backtick;
            position += 1;
            continue;
        }
        if in_backtick {
            position += 1;
            continue;
        }

        // **^mantra^** - mula mantra syntax
        // only at the START of a line (position 0) - mid-line occurrences are ignored
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
                // only create mula mantra if at start of line (position 0)
                if !mantra_text.is_empty() && position == 0 {
                    if let Some(idx) = bhasya_index {
                        // add to mantras with mula_bhasyas
                        let entry = repo.mantras.entry(mantra_text.clone()).or_default();
                        entry.mula_bhasyas.push(idx);

                        // set first definition location if not set
                        if entry.file.is_empty() {
                            entry.file = file_name.to_string();
                            entry.line = line_num;
                        }

                        // mark as explained if this is a Mula bhasya
                        let is_mula = repo.bhasyas.get(idx)
                            .map(|b| matches!(b.kind, BhasyaKind::Mula))
                            .unwrap_or(false);
                        if is_mula {
                            entry.has_explanation = true;
                        }
                    }
                }
                continue;
            }
            position += 2; // already consumed two *
            continue;
        }
        position += 1;

        // _| mantra |_ - anusrit syntax
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

                // if inside a bhasya, add to mantras.anusrit_bhasyas; otherwise to anusrits
                if let Some(idx) = bhasya_index {
                    let entry = repo.mantras.entry(ref_text).or_default();
                    entry.anusrit_bhasyas.push(idx);
                } else {
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
}

/// Parse a line outside quote blocks for anusrits only
fn parse_line_for_anusrits(
    line: &str,
    file_name: &str,
    line_num: usize,
    repo: &mut Repository,
    bhasya_index: Option<usize>,
) {
    let mut chars = line.chars().peekable();
    let mut in_backtick = false;

    while let Some(c) = chars.next() {
        if c == '`' {
            in_backtick = !in_backtick;
            continue;
        }
        if in_backtick {
            continue;
        }

        // _| mantra |_ - anusrit syntax
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

                // if inside a bhasya, add to mantras.anusrit_bhasyas; otherwise to anusrits
                if let Some(idx) = bhasya_index {
                    let entry = repo.mantras.entry(ref_text).or_default();
                    entry.anusrit_bhasyas.push(idx);
                } else {
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
