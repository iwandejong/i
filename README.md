<div align="center">

# i

**A fuzzy, recursive `cd` replacement.**
Type a few letters, land several directories deep — not just the immediate
children `cd`'s own tab-completion gives you.

[![License: MIT](https://img.shields.io/github/license/iwandejong/i)](LICENSE)
[![Release](https://img.shields.io/github/v/release/iwandejong/i)](https://github.com/iwandejong/i/releases/latest)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

```sh
curl -fsSL https://raw.githubusercontent.com/iwandejong/i/main/install.sh | sh
```

</div>

---

```sh
~/Downloads/i ❯ i test/t2
~/Downloads/i/test/t2 ❯
```

`i` is both the shell command and the Rust binary behind it — the shell
function shells out to the real `i` executable via `command i` so the two
don't recurse into each other. Inspired by the `@`-mention picker in
[pi](https://github.com/earendil-works/pi), but scoped and navigable the way
a `cd` replacement needs to be.

## Why

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

**One-liner (no Rust required)** — downloads the right prebuilt binary for
your platform (macOS x86_64/arm64, Linux x86_64), installs it to
`~/.local/bin`, and wires up shell integration for you:

```sh
curl -fsSL https://raw.githubusercontent.com/iwandejong/i/main/install.sh | sh
```

Then restart your shell (or `source ~/.zshrc`) and you're done.

<details>
<summary>Manual install, or building from source</summary>

**Prebuilt binary** — grab the tarball for your platform from the
[latest release](https://github.com/iwandejong/i/releases/latest). Each one
bundles the `i` binary plus `i.zsh` (the shell wrapper, see below):

```sh
tar xzf i-<target>.tar.gz
cp i-<target>/i ~/.local/bin/i     # put it on your $PATH
# i-<target>/i.zsh is what you'll `source` below
```

**From source** — requires Rust (stable, 1.75+):

```sh
git clone https://github.com/iwandejong/i.git
cd i
cargo build --release
cp target/release/i ~/.local/bin/i   # put it on your $PATH
```

The binary is a single static-ish executable (~1.2 MB).

</details>

## Shell integration

The `i` binary itself only *prints* matching paths to stdout — it can't
change your shell's working directory on its own (no subprocess can).
`install.sh` handles this automatically for zsh; to do it by hand:

```sh
# ~/.zshrc
source /path/to/i/shell/i.zsh
```

Then:

```sh
i              # cd $HOME, same as plain `cd`
i test         # cd to the best fuzzy match for "test" under $PWD
i ../robofuel  # scoped one level up first, then fuzzy-searching "robofuel"
i te<TAB>      # see (and pick from) the top matches before committing
i -2           # cd up 2 directories (../..)
```

(Rename the `i` function in `shell/i.zsh` to whatever you'd rather type —
`j`, `cx`, anything that isn't already taken. The binary can stay named `i`
either way; the shell function just calls it via `command i`.)

Only a zsh wrapper is included; a bash version would be nearly identical
(`i() { local d; d=$(command i "$@") && [ $? -eq 0 ] && cd "$d"; }`, minus
the completion widget).

## How it works

There's no full-screen picker UI — the `i` shell function runs the `i`
binary, which prints its best match(es) and exits; the wrapper `cd`s to
whatever came back.

- **Plain `i <query>`** — the binary prints its single top-ranked match, the
  wrapper `cd`s straight there.
- **`i <query><TAB>`** — `i --complete` prints up to 5 ranked candidates,
  and zsh's own completion system turns them into a real, cyclable menu
  (`Tab`/arrows to move through it, `Enter` accepts *and* runs in one press).
  Matched characters are highlighted in the listing as you type, even when
  they're non-contiguous in the result (true fuzzy matches, not just
  substrings).
- **A fully-typed, unambiguous path** (e.g. `i test/t2`) resolves and `cd`s
  there directly — no menu, no fuzzy noise mixed in.

<details>
<summary><strong>Config</strong></summary>

Optional JSON config at `~/.config/i/config.json` (all fields optional,
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

</details>

<details>
<summary><strong>How the path-navigation parsing works</strong></summary>

See `src/pathnav.rs` (has unit tests) — each `/`-terminated segment you've
typed is checked in order: `..` pops the root up one level, `~` (only as the
very first character) jumps to `$HOME`, a leading `/` jumps to filesystem
root, and any other segment is only consumed if it's an exact, existing
child directory of the current root — otherwise parsing stops there and
everything from that point on is treated as the fuzzy query. That's what
lets `../test` mean "go up one, then fuzzy-search test" while a query that
merely *contains* a slash-like fuzzy term doesn't get misparsed as
navigation.

</details>

<details>
<summary><strong>Project layout</strong></summary>

```
src/
  main.rs     — CLI entry (clap), wires config + walker + search together
  config.rs   — loads ~/.config/i/config.json, defaults
  pathnav.rs  — parses typed text into (resolved root, fuzzy query)
  walker.rs   — recursive directory walk with prune-list + safety caps
  search.rs   — fuzzy scoring, sorting, exact-match and depth preference
tests/
  walker_smoke.rs — confirms excludes prune subtrees and deep dirs still surface
shell/
  i.zsh       — the `i` wrapper function that actually cd's, plus completion
install.sh    — one-command installer (prebuilt binary + shell integration)
```

</details>

<details>
<summary><strong>Known limitations / good next steps</strong></summary>

- No frecency (most-recently/most-often visited dirs ranked higher) —
  everything is pure fuzzy score today. Would be a nice v2 (a small
  `~/.local/share/i/frecency.json` keyed by absolute path).
- No `.gitignore` awareness — exclusion is the flat `excludes` name list
  only. Could add real `.gitignore` parsing per-subtree later if the noise
  becomes annoying, but it adds real complexity for a tool that's meant to
  be simple and fast.
- Symlinked directories are skipped outright (avoids cycles); could
  optionally follow with cycle detection later.
- Only a zsh wrapper is included; bash/fish ports are welcome contributions.
- No prebuilt Linux arm64 binary yet — build from source there.

</details>

## Contributing

Issues and PRs welcome. `cargo test` before sending a PR; `src/pathnav.rs`
and `tests/walker_smoke.rs` are the places most behavior changes should add
a case to.

## License

[MIT](LICENSE)
