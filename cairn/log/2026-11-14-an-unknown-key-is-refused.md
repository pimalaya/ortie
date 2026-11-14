---
cairn: log
change: an-unknown-key-is-refused
landed: 2026-11-14
---

# An unknown key is refused

Ortie follows the owner's family-wide decision of 2026-11-14: a configuration key nothing reads fails loading wherever it sits, and a key an earlier Ortie read is refused by name with its replacement and a link to the notes of its removal.

## What landed

- src/config.rs: every config struct already denied unknown fields (`Config`, `AccountConfig`, `EndpointsConfig`, `StoragesConfig`, `StorageConfig`), and none used `#[serde(flatten)]`, so no struct changed shape. `AccountConfig` gains `hooks: Option<HooksRemovedConfig>`, an uninhabited enum whose `Deserialize` fails with "The `hooks` table was removed in 3.0.0: chain a command after `storage.write.command`, which receives the token on every issue and refresh, see https://github.com/pimalaya/ortie/blob/master/CHANGELOG.md", whatever the table holds. Ortie has no MIGRATION.md, so the link is the CHANGELOG. `hooks` is the one key a release read and a later one removed.
- Cargo.toml: `[patch.crates-io]` takes pimalaya-cli from ../cli (local 79c0d9e, which adds modules over the pinned 0be1b1f); Cargo.lock drops the git source.
- CHANGELOG.md: the hooks removal entry says the table is refused by name.

## Evidence

src/config.rs `a_removed_key_points_at_the_migration_notes` (a `hooks` dotted key, a nested `notify`, an empty inline table and a `[accounts.test.hooks.on-issue.success]` header) failed on the old code, which answered ``unknown field `hooks` ``; it replaces `a_hooks_table_is_refused`, which asserted that generic error. `an_unknown_key_is_refused_at_every_level` (the root, an account, `endpoints`, `storage`, `storage.read`, `storage.write`, and a second key beside `client-secret.raw`) passed on the old code, the refusal being in place since 2.0.0, and fails with `deny_unknown_fields` taken out of src/config.rs. pimalaya-config's `Secret` refuses a second key without naming it ("wanted exactly 1 element"), so that case asserts the refusal only. `the_sample_configuration_parses_with_each_variant` passes, every variant of config.sample.toml loading.

`cargo fmt --check`; `cargo test --all-features`; `cargo clippy --all-targets -- -D warnings` over the default features, `--all-features`, and `--no-default-features` with `rustls-ring`, `rustls-aws`, `native-tls` and `native-tls,vendored`: clean. `--no-default-features` alone does not build (`Oauth20ClientStd::connect` needs a TLS feature), as before this change. `cargo deny check` passes. Live suites not run.

## Capabilities moved

- config (cairn/spec/config.md): added "The configuration refuses every key it does not read"; modified "v1 config compatibility".
