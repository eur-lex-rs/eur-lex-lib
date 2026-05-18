# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`eur-lex-rs` is a Cargo workspace (edition 2024) in early development with two member crates:

- **`eur-lex-lib`** (`eur-lex-lib/`) — library that parses Formex 4 XML into typed Rust structs
- **`eur-lex-utils`** (`eur-lex-utils/`) — two CLI binaries (`eur_lex_loader`, `eur_lex_fetch`) that depend on `eur-lex-lib`

Test fixtures are in `data/` at the workspace root. Integration tests are in `eur-lex-lib/tests/` and reference fixtures via `../data/`.

## Commit messages

Each commit message must include:

1. **Instructions** — the user's request as given
2. **Changes** — what was modified and why
3. **Pros and cons** — trade-offs introduced by this approach
4. **Alternatives** (when applicable) — other solutions that could have achieved the same goal

## Commands

```bash
cargo build                        # compile all crates
cargo build -p eur-lex-lib        # compile library only
cargo build -p eur-lex-utils      # compile binaries only
cargo build --release              # release build (binaries in target/release/)
cargo test                         # run all tests across workspace
cargo test -p eur-lex-lib         # run library unit + integration tests only
cargo test <name>                  # run a single test by name
cargo clippy                       # lint all crates
cargo fmt                          # format all crates
cargo doc --open -p eur-lex-lib   # open library API docs
```
