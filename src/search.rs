use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

pub struct Scored {
    pub display: String,
    pub score: i64,
    pub depth: usize,
}

/// Fuzzy-score `entries` (all children of `root`) against `query` and sort
/// best-first: name matches rank above deep path matches, ties broken by
/// shallower then shorter.
pub fn score_and_sort(root: &std::path::Path, entries: &[PathBuf], query: &str) -> Vec<Scored> {
    let matcher = SkimMatcherV2::default().smart_case();
    let mut out: Vec<Scored> = entries
        .iter()
        .filter_map(|path| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let rel = path.strip_prefix(root).unwrap_or(path);
            let rel_str = rel.to_string_lossy();

            let score = if query.is_empty() {
                0
            } else if rel_str == query {
                // Exact path match (e.g. "cdz/test/t2") — always wins over
                // fuzzy noise, no matter how deep or how the scorer would
                // otherwise rank it.
                i64::MAX
            } else if let Some(s) = matcher.fuzzy_match(name, query) {
                s * 3
            } else {
                matcher.fuzzy_match(&rel_str, query)?
            };

            let depth = rel.components().count();
            // Nudge deeply-buried matches (cache dumps, app-support
            // ephemera, browser profile innards) below closer, more likely
            // "yours" directories when fuzzy scores are otherwise similar.
            let score = if score == i64::MAX { score } else { score - depth as i64 * 2 };
            Some(Scored {
                display: rel_str.to_string(),
                score,
                depth,
            })
        })
        .collect();

    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.depth.cmp(&b.depth))
            .then(a.display.len().cmp(&b.display.len()))
    });
    out
}
