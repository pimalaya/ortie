---
cairn: log
change: the-provider-accounts-in-the-readme
landed: 2026-11-21
---

# The provider accounts in the README

Pimalaya guideline readme-011 moved the provider recipes from config.sample.toml to the README, and readme-007 allowed the Alternatives section again.

## What reaches users

- The README's Google, Microsoft and Fastmail subsections each carry a whole account to paste (`google`, `microsoft-graph`, `microsoft-imap-smtp`, `fastmail`), with the same client, endpoints, resources and scopes the sample sections held and storage commands on `pass`.
- config.sample.toml keeps the annotated field skeleton and the custom section, pointing to the README for the providers.
- The README lists Alternatives again: pizauth, oama, mutt_oauth2.py.

## Capabilities moved

config, packaging, provider-quirks (see the change's delta).

## Verified

`cargo test` (126 passed, 8 ignored, the live suites compiled and not run). The new test `the_readme_provider_accounts_parse` failed with "README.md:202: fastmail: Parse TOML config" once the Fastmail block's `endpoints.token` was renamed `endpoints.tokn`, and passed once restored. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` under the default features, `native-tls` and `rustls-aws` alone.
