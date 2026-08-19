# i

A fuzzy, recursive `cd` replacement. Type a few letters, see matching
directories several levels deep under wherever you currently are — not just
the immediate children `cd`'s own tab-completion gives you.

```sh
~/Downloads ❯ i cdz/test/t2
~/Downloads/cdz/test/t2 ❯
```

The shell command is `i`; the engine behind it is a small Rust binary called
`cdz` (this repo's crate name, and what `i` shells out to). Inspired by the
`@`-mention picker in [pi](https://github.com/earendil-works/pi), but scoped
and navigable the way a `cd` replacement needs to be:

- **Scoped to your current directory by default.** Typing `test` searches
  recursively under `$PWD`, not your whole home directory.
- **Live path navigation while typing.** Type `../` to rescope the search to
  the parent directory (repeat for further up), `~/` to jump home, or `/` for
  filesystem root. Whatever you type *after* the last `/` is the fuzzy query
  against the resolved root. `i -2` jumps straight up 2 directories, no
  fuzzy search involved.
- **Deep, not flat.** Matches at any depth under the current root are
  shown, ranked by fuzzy score (name matches first, then full relative-path
  matches). An exact path match always wins outright; otherwise shallower
  results are preferred over deeply-buried ones (cache dumps, app-support
  ephemera) when scores are close.
- **No daemon, no whole-disk cache.** Each root you visit is walked live and
  memoized only for the current session — the tool prints matching paths
  and exits.
- **`i` alone goes home**, same as plain `cd`.

## Install

Requires Rust (stable, 1.75+).

```sh
git clone https://github.com/iwandejong/i.git
cd i
cargo build --release
cp target/release/cdz ~/.local/bin/cdz   # put it on your $PATH
```

The binary is a single static-ish executable (~1.2 MB).

## Shell integration (required)

`cdz` itself only *prints* matching paths to stdout — it can't change your
shell's working directory on its own (no subprocess can). Source the wrapper
function so it actually `cd`s for you:

```sh
# ~/.zshrc
source /path/to/i/shell/cdz.zsh
```

Then:

```sh
i              # cd $HOME, same as plain `cd`
i test         # cd to the best fuzzy match for "test" under $PWD
i ../robofuel  # scoped one level up first, then fuzzy-searching "robofuel"
i te<TAB>      # see (and pick from) the top matches before committing
i -2           # cd up 2 directories (../..)
```

(Rename the `i` function in `shell/cdz.zsh` to whatever you'd rather type —
`j`, `cx`, anything that isn't already taken.)

Only a zsh wrapper is included; a bash version would be nearly identical
(`i() { local d; d=$(cdz "$@") && [ $? -eq 0 ] && cd "$d"; }`, minus the
completion widget).

## How it works

There's no full-screen picker UI — `i` runs `cdz`, which prints its best
match(es) and exits; the shell wrapper `cd`s to whatever came back.

- **Plain `i <query>`** — `cdz` prints its single top-ranked match, the
  wrapper `cd`s straight there.
- **`i <query><TAB>`** — `cdz --complete` prints up to 5 ranked candidates,
  and zsh's own completion system turns them into a real, cyclable menu
  (`Tab`/arrows to move through it, `Enter` accepts *and* runs in one press).
  Matched characters are highlighted in the listing as you type, even when
  they're non-contiguous in the result (true fuzzy matches, not just
  substrings).
- **A fully-typed, unambiguous path** (e.g. `i cdz/test/t2`) resolves and
  `cd`s there directly — no menu, no fuzzy noise mixed in.

## Config

Optional JSON config at `~/.config/cdz/config.json` (all fields optional,
shown here with defaults):

```json
{
  "excludes": [".git", ".hg", ".svn", "node_modules", ".venv", "venv",
               "__pycache__", "target", ".cache", ".npm", ".cargo",
               ".rustup", ".Trash", "dist", "build", "proc", "sys", "dev",
               "Library"],
  "include_hidden": false,
  "max_depth": 5,
  "max_entries": 50000
}
```

- `excludes` — directory *names* that are pruned entirely: never listed,
  never descended into. Unlike `.gitignore`-based tools, this is a flat
  name list, so it applies everywhere (useful since a walk here can span
  many unrelated project roots as you go `../`).
- `include_hidden` — set `true` to also walk into dot-directories.
- `max_depth` / `max_entries` — safety caps so an accidental `~/` or `/`
  jump into a huge tree doesn't hang; you still get whatever it collected
  before hitting the cap.

## How the path-navigation parsing works

See `src/pathnav.rs` (has unit tests) — each `/`-terminated segment you've
typed is checked in order: `..` pops the root up one level, `~` (only as the
very first character) jumps to `$HOME`, a leading `/` jumps to filesystem
root, and any other segment is only consumed if it's an exact, existing
child directory of the current root — otherwise parsing stops there and
everything from that point on is treated as the fuzzy query. That's what
lets `../test` mean "go up one, then fuzzy-search test" while a query that
merely *contains* a slash-like fuzzy term doesn't get misparsed as
navigation.

## Project layout

```
src/
  main.rs     — CLI entry (clap), wires config + walker + search together
  config.rs   — loads ~/.config/cdz/config.json, defaults
  pathnav.rs  — parses typed text into (resolved root, fuzzy query)
  walker.rs   — recursive directory walk with prune-list + safety caps
  search.rs   — fuzzy scoring, sorting, exact-match and depth preference
tests/
  walker_smoke.rs — confirms excludes prune subtrees and deep dirs still surface
shell/
  cdz.zsh     — the `i` wrapper function that actually cd's, plus completion
```

## Known limitations / good next steps

- No frecency (most-recently/most-often visited dirs ranked higher) —
  everything is pure fuzzy score today. Would be a nice v2 (a small
  `~/.local/share/cdz/frecency.json` keyed by absolute path).
- No `.gitignore` awareness — exclusion is the flat `excludes` name list
  only. Could add real `.gitignore` parsing per-subtree later if the noise
  becomes annoying, but it adds real complexity for a tool that's meant to
  be simple and fast.
- Symlinked directories are skipped outright (avoids cycles); could
  optionally follow with cycle detection later.
- Only a zsh wrapper is included; bash/fish ports are welcome contributions.

## Contributing

Issues and PRs welcome. `cargo test` before sending a PR; `src/pathnav.rs`
and `tests/walker_smoke.rs` are the places most behavior changes should add
a case to.

## License

[MIT](LICENSE)
