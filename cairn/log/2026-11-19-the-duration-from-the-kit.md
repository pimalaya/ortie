---
cairn: log
change: the-duration-from-the-kit
landed: 2026-11-19
---

# The duration from the kit

Ortie moved onto pimalaya-cli 0.7.1, released on 2026-11-19, with its new `duration` feature. src/token/issued.rs `friendly` and its test `a_duration_prints_in_days_at_most` are deleted: `token inspect`, the issued-token receipt and the `auth get` device request call `pimalaya_cli::duration::friendly`, the same function moved as is, whose test holds the union of Ortie's and Comodoro's cases. jiff stays a direct dependency: src/token/inspect.rs formats instants with `civil` and `SignedDuration`.

## What reaches users

Nothing: the text output is byte-identical.

## Capabilities moved

None: output keeps its requirement *A duration prints in jiff's friendly form*.

## Verified

`cargo fmt`, `cargo test` (default and `--all-features`), `cargo clippy --locked --all-targets -- -D warnings` over the CI feature sets (default, `--all-features`, each TLS provider alone) and `cargo deny check` pass.
