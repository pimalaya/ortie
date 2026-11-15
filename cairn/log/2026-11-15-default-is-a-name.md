---
cairn: log
change: default-is-a-name
landed: 2026-11-15
---

# `default` is a name, and account resolution is coded

pimalaya-config is pinned at d5a33cba3fe9826b82f15a479043a7f79ca742ce under `[patch.crates-io]`, so `-a default` takes the account named `default` and a missing one hints to omit `-a`. `ErrorCode` (src/error.rs) gains `account-missing`, `account-default-missing` and `account-default-several`, read off pimalaya-config's `AccountError` by `error::code`; the `ortie-error` schema lists them.

Evidence: tests/codes.rs `an_account_resolution_failure_carries_its_code` and `default_is_an_account_name_like_any_other` run the binary under `--json`; both failed with the account walk disabled, and the second asserts the account named `default` is taken, which the old pin read as the marked one.

Capabilities moved: config ("Account resolution failures name what is missing"), output ("A failure carries a stable code").
