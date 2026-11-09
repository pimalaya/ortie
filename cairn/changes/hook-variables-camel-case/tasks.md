---
cairn: tasks
change: hook-variables-camel-case
---

- [ ] Proposal and delta reviewed (user): the twin command variables until 4.0
- [ ] src/account.rs: camelCase variables, the former names exported beside them to commands
- [ ] config.sample.toml and README on the new names
- [ ] Tests: each delta scenario
- [ ] `cargo fmt`, `cargo test --all-features`, `cargo clippy --all-features`, `cargo check --no-default-features --features rustls-ring`
- [ ] Fold the delta, log entry, CHANGELOG under Unreleased (**Breaking**, each rename listed), shipped in 3.0.0
