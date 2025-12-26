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

#[derive(Debug, Clone)]
pub struct Reference {
    pub mantra_text: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Repository {
    pub mantras: HashMap<String, Mantra>,
    pub references: Vec<Reference>,
}

impl Repository {
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

            // only process .vyasa files or text files for now
            let ext = file_path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("vyasa") | Some("txt") | Some("md")) {
                continue;
            }

            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue, // skip files we can't read
            };

            let file_name = file_path.to_string_lossy().to_string();
            parse_file(&content, &file_name, &mut repo);
        }

        Ok(repo)
    }

    pub fn unreferenced_mantras(&self) -> Vec<&Mantra> {
        self.mantras
            .values()
            .filter(|m| !self.references.iter().any(|r| r.mantra_text == m.text))
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
            *counts.entry(reference.mantra_text.clone()).or_insert(0) += 1;
        }
        counts
    }
}

fn parse_file(content: &str, file_name: &str, repo: &mut Repository) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // check for mantra start
        if line == "--" {
            if let Some((mantra, end_line, has_explanation)) =
                parse_mantra(&lines, i, file_name)
            {
                repo.mantras.insert(mantra.text.clone(), mantra);
                i = end_line + 1;

                // skip explanation text until next mantra or end
                if has_explanation {
                    while i < lines.len() && lines[i].trim() != "--" {
                        // look for references in explanation
                        parse_references(lines[i], file_name, i + 1, repo);
                        i += 1;
                    }
                }
                continue;
            }
        }

        // look for references in regular text
        parse_references(lines[i], file_name, i + 1, repo);
        i += 1;
    }
}

fn parse_mantra(
    lines: &[&str],
    start: usize,
    file_name: &str,
) -> Option<(Mantra, usize, bool)> {
    let mut i = start + 1;
    let mut mantra_lines = Vec::new();

    // collect mantra text until closing --
    while i < lines.len() {
        let line = lines[i].trim();
        if line == "--" {
            break;
        }
        mantra_lines.push(lines[i]);
        i += 1;
    }

    if i >= lines.len() {
        return None; // unclosed mantra
    }

    let mantra_text = mantra_lines.join("\n").trim().to_string();
    if mantra_text.is_empty() {
        return None;
    }

    // check if there's explanation after the closing -- (skip empty lines)
    let mut has_explanation = false;
    let mut j = i + 1;
    while j < lines.len() {
        let next_line = lines[j].trim();
        if next_line == "--" {
            break; // hit next mantra without explanation
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
                });
            }
        }
    }
}
