---
cairn: tasks
change: hook-variables-camel-case
---

- [x] ~~Proposal and delta reviewed (user): the twin command variables until 4.0~~ Dropped with the change: hooks-removed (2026-11-09) left no variable to rename.
- [x] ~~src/account.rs: camelCase variables, the former names exported beside them to commands~~
- [x] ~~config.sample.toml and README on the new names~~
- [x] ~~Tests: each delta scenario~~
- [x] ~~`cargo fmt`, `cargo test --all-features`, `cargo clippy --all-features`, `cargo check --no-default-features --features rustls-ring`~~
- [x] ~~Fold the delta, log entry, CHANGELOG under Unreleased (**Breaking**, each rename listed), shipped in 3.0.0~~
