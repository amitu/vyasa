use crate::parser::Repository;
use std::path::Path;

pub fn run(path: &Path, buckets: usize) -> Result<(), String> {
    let repo = Repository::parse(path)?;
    let ref_counts = repo.reference_counts();

    let total_mantras = repo.mantras.len();
    let total_references = repo.references.len();
    let total_explanations = repo.mantras.values().filter(|m| m.has_explanation).count();
    let unreferenced = repo.unreferenced_mantras().len();

    println!("vyasa repository stats");
    println!("======================\n");

    println!("mantras:      {}", total_mantras);
    println!("explanations: {}", total_explanations);
    println!("references:   {}", total_references);
    println!("unreferenced: {}", unreferenced);

    if total_mantras == 0 {
        return Ok(());
    }

    // popular mantras (>10 references)
    let popular: Vec<_> = ref_counts
        .iter()
        .filter(|(_, &count)| count > 10)
        .collect();

    if !popular.is_empty() {
        println!("\npopular mantras (>10 references):");
        let mut popular: Vec<_> = popular.into_iter().collect();
        popular.sort_by(|a, b| b.1.cmp(a.1));
        for (mantra, count) in popular {
            println!("  {:>4}x  {}", count, truncate(mantra, 50));
        }
    }

    // reference histogram
    if buckets > 0 && !ref_counts.is_empty() {
        println!("\nreference distribution:");
        print_histogram(&ref_counts, buckets);
    } else if buckets == 0 && !ref_counts.is_empty() {
        println!("\nreferences per mantra:");
        let mut counts: Vec<_> = ref_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));
        for (mantra, count) in counts {
            println!("  {:>4}x  {}", count, truncate(mantra, 50));
        }
    }

    Ok(())
}

fn print_histogram(ref_counts: &std::collections::HashMap<String, usize>, max_buckets: usize) {
    let max_refs = *ref_counts.values().max().unwrap_or(&0);
    if max_refs == 0 {
        return;
    }

    // use at most max_buckets, but fewer if range is smaller
    let num_buckets = max_refs.min(max_buckets);
    let bucket_size = (max_refs + num_buckets - 1) / num_buckets;
    let mut bucket_counts = vec![0usize; num_buckets];

    for &count in ref_counts.values() {
        let bucket = (count.saturating_sub(1)) / bucket_size;
        let bucket = bucket.min(num_buckets - 1);
        bucket_counts[bucket] += 1;
    }

    // find first and last non-empty buckets
    let first_non_empty = bucket_counts.iter().position(|&c| c > 0).unwrap_or(0);
    let last_non_empty = bucket_counts.iter().rposition(|&c| c > 0).unwrap_or(0);

    let max_bucket = *bucket_counts[first_non_empty..=last_non_empty]
        .iter()
        .max()
        .unwrap_or(&0);
    let bar_width = 40;

    for i in first_non_empty..=last_non_empty {
        let count = bucket_counts[i];
        let start = i * bucket_size + 1;
        let end = (i + 1) * bucket_size;
        let bar_len = if max_bucket > 0 {
            (count * bar_width) / max_bucket
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        println!("  {:>3}-{:<3} refs: {:>4} {}", start, end, count, bar);
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
