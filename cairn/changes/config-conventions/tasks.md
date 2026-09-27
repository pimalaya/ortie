---
cairn: tasks
change: config-conventions
---

# Tasks

- [x] Make `TlsConfig` public, give it an `Auto` default variant and document every variant.
- [x] Replace `impl From<TlsConfig> for Tls` and the `tls` deserializer with `TlsConfig::into_tls(self, alpn)`.
- [x] Hold `TlsConfig` in `AccountConfig` and convert once in `Account::from`.
- [x] Add the `alpn` account option, empty by default, documented as the middlebox override.
- [x] Add the `is_default` helper to `config.rs` and use it on the wizard fragment's `default`.
- [x] Leave a TODO on `opt_shell_expanded_path` naming the shared pimalaya-config helper.
- [x] Tests: an account naming both folds them into one handle, an account naming neither offers no ALPN.
- [x] config.sample.toml: the `alpn` key, and `auto` among the `tls` values.
- [x] Changelog: Added for `alpn` and for `tls = "auto"`.
- [x] Build, test, fmt and clippy clean; the live configuration still loads.
- [x] Fold into cairn/spec/config.md; log; land.
