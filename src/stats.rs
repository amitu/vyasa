use crate::parser::{find_repo_root, Repository};
use crate::snapshot::{compare_with_canon, Canon, CanonSearchResult, MantraStatus};
use std::path::Path;

pub fn run(path: &Path, buckets: usize) -> Result<(), String> {
    let repo = Repository::parse(path)?;
    let ref_counts = repo.reference_counts();

    let total_mantras = repo.mantras.len();
    let total_references = repo.references.len();
    let total_explanations = repo.mantras.values().filter(|m| m.has_explanation).count();
    let unreferenced = repo.unreferenced_mantras().len();

    // load canon for stats
    let repo_root = find_repo_root(path);
    let canon = repo_root.as_ref().and_then(|r| match Canon::find(r) {
        CanonSearchResult::Found(c) => Some(c),
        _ => None,
    });

    println!("vyasa repository stats");
    println!("======================\n");

    println!("mantras:      {}", total_mantras);
    println!("explanations: {}", total_explanations);
    println!("references:   {}", total_references);
    println!("unreferenced: {}", unreferenced);

    // canon stats
    if let Some(ref canon) = canon {
        let mantras_with_status = compare_with_canon(&repo, canon);
        let accepted = mantras_with_status
            .iter()
            .filter(|m| matches!(m.status, MantraStatus::Accepted))
            .count();
        let new = mantras_with_status
            .iter()
            .filter(|m| matches!(m.status, MantraStatus::New))
            .count();
        let changed = mantras_with_status
            .iter()
            .filter(|m| matches!(m.status, MantraStatus::Changed { .. }))
            .count();
        let orphaned = mantras_with_status
            .iter()
            .filter(|m| matches!(m.status, MantraStatus::OrphanedInCanon { .. }))
            .count();

        println!("\ncanon:");
        println!("  accepted:   {}", accepted);
        println!("  pending:    {}", new);
        if changed > 0 {
            println!("  changed:    {}", changed);
        }
        if orphaned > 0 {
            println!("  orphaned:   {}", orphaned);
        }
    } else {
        println!("\ncanon:        (no canon.md found)");
    }

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
    if buckets > 0 {
        println!("\nreference distribution:");
        print_histogram(&ref_counts, buckets, total_mantras);
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

fn print_histogram(
    ref_counts: &std::collections::HashMap<String, usize>,
    max_buckets: usize,
    total_mantras: usize,
) {
    // count of mantras with 0 references
    let zero_refs = total_mantras - ref_counts.len();

    let max_refs = *ref_counts.values().max().unwrap_or(&0);

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

    for &count in ref_counts.values() {
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
            println!("  {:>3}     refs: {:>4} {}", start, count, bar);
        } else {
            println!("  {:>3}-{:<3} refs: {:>4} {}", start, end, count, bar);
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
