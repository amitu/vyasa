use crate::parser::Repository;
use std::path::Path;

// [vyasa check exits with non zero exit code if any rule is violated]
pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let unexplained = repo.unexplained_mantras();
    let spacing_violations = &repo.spacing_violations;

    let mut has_errors = false;

    // check for unexplained mantras
    if !unexplained.is_empty() {
        has_errors = true;
        println!(
            "found {} mantras without explanations:\n",
            unexplained.len()
        );
        for mantra in &unexplained {
            println!("  {}:{}", mantra.file, mantra.line);
            println!("    {}\n", truncate(&mantra.text, 60));
        }
    }

    // [before / after -- must contain at least one empty line, unless at start/end of file]
    if !spacing_violations.is_empty() {
        has_errors = true;
        println!(
            "found {} spacing violations:\n",
            spacing_violations.len()
        );
        for violation in spacing_violations {
            println!("  {}:{}", violation.file, violation.line);
            println!("    {}\n", violation.message);
        }
    }

    if has_errors {
        let mut errors = Vec::new();
        if !unexplained.is_empty() {
            errors.push(format!("{} unexplained mantras", unexplained.len()));
        }
        if !spacing_violations.is_empty() {
            errors.push(format!("{} spacing violations", spacing_violations.len()));
        }
        Err(errors.join(", "))
    } else {
        println!("all {} mantras validated successfully", repo.mantras.len());
        Ok(())
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else if s.contains('\n') {
        format!("{}...", first_line)
    } else {
        first_line.to_string()
    }
}
