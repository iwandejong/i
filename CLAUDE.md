# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`i` is a fuzzy, recursive `cd` replacement: a small Rust CLI (crate/binary
name `i`) plus a zsh wrapper (`shell/i.zsh`) that turns it into an actual
shell command. The binary only ever prints matching paths to stdout and
exits — it cannot change the shell's working directory itself, which is why
the zsh wrapper exists and is required for the tool to be usable at all.

## Commands

```sh
cargo build --release   # binary at target/release/i
cargo test               # all unit tests + tests/walker_smoke.rs
cargo test config::      # run one module's tests (config::, search::, walker::, pathnav::)
cargo test full_json_round_trips   # run a single test by name
cargo fmt                # required clean before a PR
cargo clippy --all-targets   # required clean before a PR (no warnings currently)
```

No other build tooling — pure `cargo`, Rust stable 1.75+.

Manual end-to-end check after a change (no test harness drives the shell
wrapper itself):

```sh
zsh -ic 'source shell/i.zsh; cd /some/test/tree; i somequery'
```

## Architecture

Two halves that only communicate through stdin/stdout/exit-code, never a
shared library boundary:

1. **`src/*.rs`** — the `i` binary. Stateless: every invocation walks the
   filesystem fresh and prints 0, 1, or up to 5 matching paths, one per
   line, then exits. No daemon, no persistent cache, no in-process state
   between calls.
2. **`shell/i.zsh`** — defines the `i` shell function (what the user
   actually types) and the `_i` zsh completion widget. Both call the `i`
   binary via `command i ...` (bypassing the shell function itself, since
   they share a name) and `cd` to whatever line(s) come back.

### Request flow through `src/main.rs`

For a typed query string, in order:

1. `pathnav::resolve(cwd, query)` walks any leading `../`, `~/`, `/`, or
   exact-existing-child-dir segments and returns `(resolved_root,
   remaining_fuzzy_query)`. This is what lets `../foo` mean "go up one, then
   fuzzy-search foo" — see the doc comment on `pathnav::resolve` for the
   full segment-consumption rules and edge cases.
2. `walker::build_index(resolved_root, ...)` recursively collects all
   subdirectories under that root, pruning by the config's `excludes` name
   list, `include_hidden`, `max_depth`, and `max_entries` — this is a fresh
   walk every single call, no caching.
3. `search::score_and_sort(...)` fuzzy-scores every collected path against
   the remaining query and sorts best-first. Three ranking rules layered on
   top of raw fuzzy score, in priority order: an exact relative-path match
   always wins outright (`i64::MAX`, checked before scoring); a match on the
   leaf directory *name* is weighted 3x over a match that only hits deeper
   path segments; and among comparable scores, shallower/less-buried
   results are nudged ahead via a small per-depth-level penalty.
4. `main.rs` decides how many results to print: 1 for a plain `cd` call, up
   to 5 for `--complete` (tab-completion), but always just 1 if the top
   result was an exact match — no point cluttering completion with fuzzy
   noise when the query was already unambiguous.
5. If nothing scored (e.g. the resolved root has no subdirectories left to
   search) but the query was empty, it falls back to printing the resolved
   root itself rather than failing — so a fully-typed path like `foo/bar/`
   still lands you at `bar` even if `bar` has no children.

### Config

`src/config.rs` loads optional JSON from `Config::config_path()` (platform
config dir + `i/config.json` — `~/Library/Application Support/i/` on macOS,
`~/.config/i/` on Linux, despite what casual docs might say for the
Linux-only path). The struct is `#[serde(default)]`, so a config file
overriding just one field must not lose the rest of the defaults — this is
covered by `config::tests::partial_json_fills_missing_fields_from_defaults`
and is easy to silently break by removing that attribute.

### Shell integration (`shell/i.zsh`)

Non-obvious pieces worth knowing before touching this file:

- The `_i` completion widget builds a **multi-group glob pattern** (one
  parenthesized capture group per typed character) for zsh's `list-colors`,
  rather than a single contiguous-substring pattern — because fuzzy matches
  are frequently non-contiguous, and a naive substring pattern would leave
  most candidates unhighlighted.
- `compstate[insert]=menu` after `compadd` is required: without it, zsh's
  first Tab tries to insert the longest common prefix across all candidates
  before showing a menu, which is empty (and looks like the typed text got
  wiped) since fuzzy candidates rarely share a prefix.
- `bindkey -M menuselect '^M' .accept-line` makes one Enter both accept the
  highlighted menu candidate and run the command — vanilla zsh needs two
  Enters (accept, then run) without this.
- The wrapper function and the binary are both named `i`; calls into the
  binary go through `command i` specifically to avoid the shell function
  recursing into itself.

## Testing conventions

Every module (`config.rs`, `pathnav.rs`, `search.rs`, `walker.rs`) has its
own `#[cfg(test)] mod tests` at the bottom of the file — add cases there,
not in a separate test file, unless the test needs to exercise the compiled
binary or shell wrapper end-to-end (that's what `tests/walker_smoke.rs` is
for). `walker.rs` tests build fixture trees under `std::env::temp_dir()`
keyed by test name so they can run in parallel without clobbering each
other.

Two things worth knowing before changing `walker.rs` or `search.rs`
semantics, since they're easy to get backwards:

- `max_depth: Some(N)` in the walker is **not** a strict path-depth cap —
  because the depth check happens once per recursive `walk()` call (which
  lists an entire directory's children at once) rather than per pushed
  entry, `max_depth: Some(1)` actually admits paths 2 components deep, not
  1. See `walker::tests::max_depth_caps_recursion` for the exact boundary.
- In `search.rs`, the per-depth ranking penalty is applied to *every*
  score, including the `0` baseline for an empty query — so don't assume an
  empty-query `Scored.score` stays `0`; only the `i64::MAX` exact-match
  sentinel is exempt.
