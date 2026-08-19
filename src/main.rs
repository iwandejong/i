mod config;
mod pathnav;
mod search;
mod walker;

use clap::Parser;
use config::Config;
use std::path::PathBuf;
use std::process::ExitCode;

/// i — fuzzy, recursive directory jumper.
///
/// Fuzzy-searches recursively under the current directory (or a navigated
/// root — "../" to go up, "~/" for home, "/" for filesystem root) and
/// prints matching paths, one per line, best match first. A shell wrapper
/// uses this to `cd` into the top match, or to drive tab-completion.
#[derive(Parser, Debug)]
#[command(name = "i", version)]
struct Args {
    /// Fuzzy query, e.g. `i ../robofuel`
    query: Option<String>,

    /// Print up to 5 matches instead of just the best one. Used for shell
    /// tab-completion.
    #[arg(long)]
    complete: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let cfg = Config::load();

    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("i: couldn't read current directory: {e}");
            return ExitCode::FAILURE;
        }
    };

    let initial_query = args.query.unwrap_or_default();

    let (root, query) = pathnav::resolve(&cwd, &initial_query);
    // `query` is always a trailing slice of `initial_query` (pathnav only
    // ever advances a pointer through it), so this recovers whatever
    // navigation prefix (e.g. "../") was typed before the fuzzy part.
    let nav_prefix = &initial_query[..initial_query.len() - query.len()];
    let entries = walker::build_index(
        &root,
        &cfg.excludes,
        cfg.include_hidden,
        cfg.max_depth,
        cfg.max_entries,
    );
    let scored = search::score_and_sort(&root, &entries, &query);
    // An exact path match (score == i64::MAX) is unambiguous — don't clutter
    // completion with unrelated fuzzy results alongside it.
    let limit = if scored.first().is_some_and(|s| s.score == i64::MAX) {
        1
    } else if args.complete {
        5
    } else {
        1
    };
    let matches: Vec<String> = scored
        .into_iter()
        .take(limit)
        .map(|s| format!("{nav_prefix}{}", s.display))
        .collect();

    if matches.is_empty() {
        // Nothing left to fuzzy-search (e.g. the resolved root has no
        // subdirectories) — land on the root itself rather than failing,
        // so a fully-typed path like "test/t2/" still cd's there.
        if query.is_empty() {
            println!("{nav_prefix}");
            return ExitCode::SUCCESS;
        }
        return ExitCode::from(1);
    }
    for m in matches {
        println!("{m}");
    }
    ExitCode::SUCCESS
}

// Re-exported for the pathnav unit tests to reach dirs::home_dir() etc.
#[allow(unused)]
fn _touch(_: PathBuf) {}
