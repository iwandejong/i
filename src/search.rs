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
                // Exact path match (e.g. "test/t2") — always wins over
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
            let score = if score == i64::MAX {
                score
            } else {
                score - depth as i64 * 2
            };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn empty_query_keeps_all_entries_shallowest_first() {
        let root = Path::new("/root");
        let entries = vec![PathBuf::from("/root/b/c"), PathBuf::from("/root/a")];
        let out = score_and_sort(root, &entries, "");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].display, "a");
    }

    #[test]
    fn exact_path_match_always_ranks_first() {
        let root = Path::new("/root");
        let entries = vec![
            PathBuf::from("/root/testing/utils"), // strong fuzzy match on "test"
            PathBuf::from("/root/test"),          // exact match
        ];
        let out = score_and_sort(root, &entries, "test");
        assert_eq!(out[0].display, "test");
        assert_eq!(out[0].score, i64::MAX);
    }

    #[test]
    fn name_match_beats_full_path_match() {
        let root = Path::new("/root");
        // "test" only appears in the leaf name of the first, and only in an
        // ancestor segment of the second — the name match should win.
        let entries = vec![
            PathBuf::from("/root/a/b/test"),
            PathBuf::from("/root/test/a/b/nope"),
        ];
        let out = score_and_sort(root, &entries, "test");
        assert_eq!(out[0].display, "a/b/test");
    }

    #[test]
    fn shallower_match_preferred_when_scores_are_close() {
        let root = Path::new("/root");
        // Same filename at two depths — the shallower one should sort first.
        let entries = vec![
            PathBuf::from("/root/a/b/c/d/target"),
            PathBuf::from("/root/target"),
        ];
        let out = score_and_sort(root, &entries, "target");
        assert_eq!(out[0].display, "target");
    }

    #[test]
    fn non_matching_entries_are_dropped() {
        let root = Path::new("/root");
        let entries = vec![PathBuf::from("/root/xyz")];
        let out = score_and_sort(root, &entries, "qqq");
        assert!(out.is_empty());
    }
}
