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

## Usage

```sh
i              # cd $HOME, same as plain `cd`
i test         # cd to the best fuzzy match for "test" under $PWD
i ../robofuel  # scoped one level up first, then fuzzy-searching "robofuel"
i te<TAB>      # cyclable menu of the top matches, non-contiguous chars highlighted
i -2           # cd up 2 directories (../..)
```

- **Scoped to `$PWD`**, not your whole home directory — and matches at any
  depth, not just immediate children.
- **Live navigation while typing**: `../` rescopes to the parent (repeat for
  further up), `~/` jumps home, `/` jumps to filesystem root. Whatever
  follows the last `/` is the fuzzy query.
- **Ranked sensibly**: an exact path match always wins outright; otherwise
  name matches beat full-path matches, and shallower results beat
  deeply-buried ones (cache dumps, app-support ephemera) when scores are close.
- **No daemon, no whole-disk cache** — each root is walked live, per query.

There's no full-screen picker UI: the `i` binary just prints its best
match(es) and exits, and the shell wrapper `cd`s to whatever came back
(or hands zsh a real completion menu for `<TAB>`). A fully-typed,
unambiguous path resolves and `cd`s there directly, no fuzzy noise mixed in.

## Install

**One-liner (no Rust required)** — downloads the right prebuilt binary
(macOS x86_64/arm64, Linux x86_64), installs it to `~/.local/bin`, and wires
up zsh integration:

```sh
curl -fsSL https://raw.githubusercontent.com/iwandejong/i/main/install.sh | sh
```

Restart your shell (or `source ~/.zshrc`) and you're done.

<details>
<summary>Manual install, or building from source</summary>

**Prebuilt binary** — grab the tarball for your platform from the
[latest release](https://github.com/iwandejong/i/releases/latest). Each one
bundles the `i` binary plus `i.zsh` (the shell wrapper):

```sh
tar xzf i-<target>.tar.gz
cp i-<target>/i ~/.local/bin/i   # i-<target>/i.zsh is what you'll `source` below
```

**From source** — requires Rust (stable, 1.75+):

```sh
git clone https://github.com/iwandejong/i.git && cd i
cargo build --release
cp target/release/i ~/.local/bin/i
```

</details>

## Shell integration

The `i` binary only *prints* matching paths — it can't change your shell's
directory itself. `install.sh` wires this up automatically; by hand:

```sh
# ~/.zshrc
source /path/to/i/shell/i.zsh
```

Rename the `i` function in `shell/i.zsh` to whatever you'd rather type — the
binary can stay named `i` either way, since the wrapper calls it via
`command i`. Only zsh is supported; a bash version would be nearly identical
(`i() { local d; d=$(command i "$@") && cd "$d"; }`, minus completion).

<details>
<summary><strong>Config</strong></summary>

Optional JSON config at `~/.config/i/config.json` (all fields optional,
shown with defaults):

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

- `excludes` — directory *names* pruned entirely (never listed, never
  descended into), applied everywhere a walk can reach as you go `../`.
- `include_hidden` — set `true` to also walk into dot-directories.
- `max_depth` / `max_entries` — safety caps so `~/` or `/` on a huge tree
  doesn't hang; you still get whatever it collected before hitting the cap.

</details>

<details>
<summary><strong>How path navigation is parsed</strong></summary>

See `src/pathnav.rs` (has unit tests) — each `/`-terminated segment is
checked in order: `..` pops the root up one level, `~` (only as the very
first character) jumps to `$HOME`, a leading `/` jumps to filesystem root,
and any other segment is consumed only if it's an exact, existing child
directory — otherwise parsing stops and everything from there on is the
fuzzy query. That's what lets `../test` mean "go up one, then fuzzy-search
test" without a query that merely *contains* a slash getting misparsed as
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
  walker_smoke.rs — confirms excludes prune subtrees, deep dirs still surface
shell/
  i.zsh       — the `i` wrapper function that actually cd's, plus completion
install.sh    — one-command installer (prebuilt binary + shell integration)
```

</details>

<details>
<summary><strong>Known limitations</strong></summary>

- No frecency ranking (most-recently/often visited dirs) — pure fuzzy score
  today.
- No `.gitignore` awareness — exclusion is the flat `excludes` list only.
- Symlinked directories are skipped outright (avoids cycles).
- No prebuilt Linux arm64 binary yet — build from source there.
- Only zsh ships; bash/fish ports are welcome contributions.

</details>

## Contributing

Issues and PRs welcome — run `cargo test` first.

## License

[MIT](LICENSE)
