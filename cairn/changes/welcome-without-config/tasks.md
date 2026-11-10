---
cairn: tasks
change: welcome-without-config
---

- [x] `ConfigMissingError`, `welcome` and `meet_bare_invocation` (src/cli.rs); `shell_single_quote` (src/auth/get.rs) shared
- [x] `main` prints the welcome and exits 1 for a person, `ErrorReport` otherwise
- [x] Tests: the missing configuration downcasts with its path, the welcome commands quote a path holding a space and a quote
- [x] Welcome moved to downloading config.sample.toml (providers-in-the-sample); its `curl --create-dirs -o` run into a scratch home
- [x] Seen under a pseudo-terminal on a scratch home: bare `ortie`, `-c <missing> token show`; piped and `--json` keep the error; bare with a configuration shows the help
- [x] CHANGELOG, src/main.rs header
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
