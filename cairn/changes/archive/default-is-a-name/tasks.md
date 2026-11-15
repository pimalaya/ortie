---
cairn: tasks
change: default-is-a-name
---

- [x] Cargo.toml `[patch.crates-io]`: pimalaya-config at d5a33cba3fe9826b82f15a479043a7f79ca742ce; Cargo.lock moves only its source line
- [x] src/error.rs: the three account codes, `account_code` walked by `code`
- [x] src/json_schema.rs: `the_error_schema_names_the_codes` checks `account-default-several`
- [x] tests/codes.rs: `an_account_resolution_failure_carries_its_code`, `default_is_an_account_name_like_any_other` (both fail without the code walk or on the old pin)
- [x] No doc, sample or test of Ortie wrote `-a default`
- [x] Spec folded (config, output), CHANGELOG, MIGRATION.md
- [x] `cargo fmt`, tests, clippy (both feature sets), `cargo deny check`
