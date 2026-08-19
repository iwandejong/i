# Contributing

Issues and PRs welcome.

## Setup

```sh
git clone https://github.com/iwandejong/i.git && cd i
cargo build
cargo test
```

Requires Rust (stable, 1.75+). No other tooling needed.

## Before sending a PR

```sh
cargo test
cargo fmt
cargo clippy
```

Every module (`config.rs`, `pathnav.rs`, `search.rs`, `walker.rs`) has a
`#[cfg(test)] mod tests` block at the bottom — that's where a behavior
change should add or update a case. `tests/walker_smoke.rs` is a higher-level
smoke test that exercises the walker against a real temp directory tree.

## Where things live

See the "Project layout" section in the [README](README.md) for what each
file does. Rough guide to where a change likely belongs:

- Ranking/sorting behavior → `src/search.rs`
- How typed text like `../foo` or `~/bar` gets parsed → `src/pathnav.rs`
- Directory-walking, excludes, depth/entry caps → `src/walker.rs`
- Config file loading/defaults → `src/config.rs`
- The `i` shell function, completion, keybindings → `shell/i.zsh`
- CLI wiring, argument parsing → `src/main.rs`

## Scope

This is a small, deliberately simple tool — no daemon, no cache, no
`.gitignore` parsing. New features are welcome, but prefer the smallest
change that solves the problem over new abstractions or config knobs. If
you're unsure whether something fits, open an issue first.
