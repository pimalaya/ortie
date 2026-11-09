---
cairn: tasks
change: dependencies-trimmed
---

# Tasks

## Manifest

- [x] Cargo.toml: `default-features = false` on every dependency with its features named; clap without `color` and `wrap_help`; `idna_adapter = "=1.0.0"`; open, notify-rust, dirs and shellexpand removed; the `notify` feature and its `vendored` forward removed
- [x] Dev-dependencies: env_logger, rustls and rustls-platform-verifier removed, tempfile without `getrandom`
- [x] pimalaya-cli pinned by `[patch.crates-io]` to 65d6d30 (logs-and-help-stay-plain, pushed 2026-11-09)
- [x] `cargo tree -e normal,dev` holds none of anstream, terminal_size, open, is-wsl, notify-rust, icu_normalizer, aws-lc-rs, and clap_builder has no `color`

## Code

- [x] src/auth/get.rs: no browser launch; the authorization text says to open the link, the device text prefers the complete verification URI
- [x] src/config.rs and src/account.rs: `notify` tables, `NotifyConfig` and `notify_with` removed, hooks command-only
- [x] A test: a non-ASCII host is refused, its Punycode form accepted; seen failing on the ICU4X back end
- [x] A test: a hook `notify` table is refused

## Docs and packaging

- [x] package.nix: the `notify` and dbus branches removed
- [x] config.sample.toml hooks, README Hooks bullet, CONTRIBUTING features sentence
- [x] CHANGELOG Unreleased
- [x] hook-variables-camel-case: the notification parts dropped from its proposal, tasks and delta
- [x] pimalaya/.github GUIDELINES.md cli-008: `notify` optional per product

## Checks and landing

- [x] fmt, clippy `-D warnings`, `cargo test --all-features`, `cargo check --no-default-features --features rustls-ring`, `cargo check --features native-tls`, `cargo deny check`
- [x] Release binary size and crate count, recorded in the log
- [x] Delta folded, log written, status landed
