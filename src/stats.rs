use crate::parser::Repository;
use std::path::Path;

pub fn run(path: &Path) -> Result<(), String> {
    let repo = Repository::parse(path)?;

    let total_mantras = repo.mantras.len();
    let total_anusrits = repo.anusrits.len();

    // count bhasya types
    let mut bhasya_count = 0;
    let mut uddhrit_count = 0;
    let mut khandita_count = 0;
    let mut tyakta_count = 0;

    for bhasya in &repo.bhasyas {
        if bhasya.is_deprecated {
            tyakta_count += 1;
        } else if bhasya.shastra.is_some() {
            uddhrit_count += 1;
        } else if bhasya.khandita.is_some() {
            khandita_count += 1;
        } else {
            bhasya_count += 1;
        }
    }

    println!("mantras:  {}", total_mantras);
    println!("anusrits: {}", total_anusrits);
    println!();
    println!("bhasyas:  {}", bhasya_count);
    println!("uddhrit:  {}", uddhrit_count);
    println!("khandita: {}", khandita_count);
    println!("tyakta:   {}", tyakta_count);

    Ok(())
}
