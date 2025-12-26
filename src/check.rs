use crate::parser::Repository;
use std::path::Path;

pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let unexplained = repo.unexplained_mantras();

    if unexplained.is_empty() {
        println!("all {} mantras have explanations", repo.mantras.len());
        Ok(())
    } else {
        println!(
            "found {} mantras without explanations:\n",
            unexplained.len()
        );
        for mantra in &unexplained {
            println!("  {}:{}", mantra.file, mantra.line);
            println!("    {}\n", truncate(&mantra.text, 60));
        }
        Err(format!(
            "{} mantras missing explanations",
            unexplained.len()
        ))
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
