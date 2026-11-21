---
cairn: log
change: the-readme-shows-a-terminal-block
landed: 2026-11-21
---

# The README shows a terminal block

The README shows, under its one-sentence description, one real command line, `ortie --json token inspect`, and the JSON it prints, as readme-006 has himalaya and neverest do. The guidelines were amended the same day so that any CLI may carry such a block, held to the same test.

## What landed

- README.md: an `sh` block holding the command line and a `json` block holding its output, a token issued at 2026-11-21T08:00:00Z for an hour, with a refresh token and two Google scopes.
- tests/docs.rs: `the_readme_header_block_prints_what_it_shows` runs the README's command line against a stored token matching the block and compares the printed JSON with the one shown. A one-second drift of `expiresAt` in the README fails it.

## Spec

No capability moved: the README is documentation, and `token inspect` is unchanged.

## Verified

`cargo test --test docs` (1 passed, failing on a drifted block), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.
