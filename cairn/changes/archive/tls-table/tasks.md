---
cairn: tasks
change: tls-table
---

# Tasks

- [x] src/config.rs: `TlsConfig` table (`TlsProviderConfig`, `RustlsConfig`, `RustlsCryptoConfig`), `tls_table` refusing the string by name, `HooksRemoved` through `removed_key!`, `StorageConfig` on `CommandConfig`
- [x] src/account.rs: the storage commands as `CommandConfig`, built when run
- [x] config.sample.toml: the `tls.*` lines, all commented, each the build's default when omitted
- [x] Tests: `tls_and_alpn_fold_into_one_handle` (with `tls.cert`), `the_tls_string_is_refused_naming_its_keys` (fails on the string enum), `a_storage_command_takes_the_cmd_alias`, the hooks sentence; tests/account.rs `the_tls_string_is_refused_by_name` through the binary
- [x] Spec: config; CHANGELOG, MIGRATION.md (Configuration)
