---
cairn: tasks
change: config-flag-repeated
---

- [x] `ConfigPathsArg` (src/cli.rs): no `env`, no `value_delimiter`; `a_config_path_is_never_split` seen failing against the `:` delimiter (`["C", "\\a.toml", "b", "c;d.toml"]`)
- [x] clap's `env` feature dropped from Cargo.toml
- [x] `invocation` (src/auth/get.rs) prints one `-c` per path, its test updated
- [x] README Configuration, `--config` help, CHANGELOG
- [x] .github/GUIDELINES.md: cli-009 added, readme-010 template updated
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
