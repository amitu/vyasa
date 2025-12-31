use crate::parser::{Repository, BhasyaKind};
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
        match &bhasya.kind {
            BhasyaKind::Tyakta => tyakta_count += 1,
            BhasyaKind::Uddhrit(_) => uddhrit_count += 1,
            BhasyaKind::Khandita(_) => khandita_count += 1,
            BhasyaKind::Mula => bhasya_count += 1,
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
