use crate::parser::Repository;
use std::path::Path;

pub fn run(path: &Path, buckets: usize) -> Result<(), String> {
    let repo = Repository::parse(path)?;
    let anusrit_counts = repo.anusrit_counts();

    let total_mantras = repo.mantras.len();
    let total_anusrits = repo.anusrits.len();
    let unreferenced = repo.unreferenced_mantras().len();

    // count bhasya types
    let mut bhasya_count = 0;
    let mut uddhrit_count = 0;
    let mut tyakta_count = 0;

    for bhasya in &repo.bhasyas {
        if bhasya.is_deprecated {
            tyakta_count += 1;
        } else if bhasya.shastra.is_some() {
            uddhrit_count += 1;
        } else {
            bhasya_count += 1;
        }
    }

    println!("vyasa repository stats");
    println!("======================\n");

    // mantras and anusrits (definitions and usages)
    println!("mantras:      {}", total_mantras);
    println!("anusrits:     {}", total_anusrits);
    println!("unreferenced: {}", unreferenced);

    println!();

    // bhasyas (commentaries)
    println!("bhasyas:      {}", bhasya_count);
    println!("uddhrit:      {}", uddhrit_count);
    println!("tyakta:       {}", tyakta_count);

    if total_mantras == 0 {
        return Ok(());
    }

    // popular mantras (>10 anusrits)
    let popular: Vec<_> = anusrit_counts
        .iter()
        .filter(|(_, &count)| count > 10)
        .collect();

    if !popular.is_empty() {
        println!("\npopular mantras (>10 anusrits):");
        let mut popular: Vec<_> = popular.into_iter().collect();
        popular.sort_by(|a, b| b.1.cmp(a.1));
        for (mantra, count) in popular {
            println!("  {:>4}x  {}", count, truncate(mantra, 50));
        }
    }

    // anusrit histogram
    if buckets > 0 {
        println!("\nanusrit distribution:");
        print_histogram(&anusrit_counts, buckets, total_mantras);
    } else if buckets == 0 && !anusrit_counts.is_empty() {
        println!("\nanusrits per mantra:");
        let mut counts: Vec<_> = anusrit_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));
        for (mantra, count) in counts {
            println!("  {:>4}x  {}", count, truncate(mantra, 50));
        }
    }

    Ok(())
}

fn print_histogram(
    anusrit_counts: &std::collections::HashMap<String, usize>,
    max_buckets: usize,
    total_mantras: usize,
) {
    // count of mantras with 0 anusrits
    let zero_refs = total_mantras - anusrit_counts.len();

    let max_refs = *anusrit_counts.values().max().unwrap_or(&0);

    // use at most max_buckets-1 for non-zero refs (reserve one for 0)
    let effective_buckets = if max_refs > 0 { max_buckets - 1 } else { 0 };
    let num_buckets = max_refs.min(effective_buckets).max(1);
    let bucket_size = if max_refs > 0 {
        (max_refs + num_buckets - 1) / num_buckets
    } else {
        1
    };

    // bucket_counts[0] is for 0 refs, bucket_counts[1..] for 1+ refs
    let mut bucket_counts = vec![0usize; num_buckets + 1];
    bucket_counts[0] = zero_refs;

    for &count in anusrit_counts.values() {
        if count > 0 {
            let bucket = ((count - 1) / bucket_size) + 1;
            let bucket = bucket.min(num_buckets);
            bucket_counts[bucket] += 1;
        }
    }

    // find last non-empty bucket (always show 0 refs if present)
    let first_non_empty = if zero_refs > 0 { 0 } else { 1 };
    let last_non_empty = bucket_counts.iter().rposition(|&c| c > 0).unwrap_or(0);

    let max_bucket = *bucket_counts[first_non_empty..=last_non_empty]
        .iter()
        .max()
        .unwrap_or(&0);
    let bar_width = 40;

    for i in first_non_empty..=last_non_empty {
        let count = bucket_counts[i];
        let (start, end) = if i == 0 {
            (0, 0)
        } else {
            let start = (i - 1) * bucket_size + 1;
            let end = i * bucket_size;
            (start, end)
        };
        let bar_len = if max_bucket > 0 {
            (count * bar_width) / max_bucket
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        if start == end {
            println!("  {:>3}: {:>4} {}", start, count, bar);
        } else {
            println!("  {:>3}-{:<3}: {:>4} {}", start, end, count, bar);
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else {
        first_line.to_string()
    }
}
