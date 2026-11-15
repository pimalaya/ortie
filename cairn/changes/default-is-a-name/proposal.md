---
cairn: change
id: default-is-a-name
status: landed
created: 2026-11-15
---

# `default` is a name, and account resolution is coded

## Why

pimalaya-config read `-a default` as the account marked `default = true`, so an account named `default` was unreachable whenever another one was marked. Its master d5a33cba3fe9826b82f15a479043a7f79ca742ce (owner ruling 2026-11-15, item 24) reads `default` as a name like any other: an absent or empty name takes the marked account, and a missing account named `default` hints to omit `-a`. The three ways an account fails to resolve also printed `"code": null` under `--json`, leaving a script to match the wording.

## What

- **BREAKING**: `-a default` takes the account named `default`; Ortie gets it by pinning pimalaya-config to d5a33cba3fe9826b82f15a479043a7f79ca742ce under `[patch.crates-io]`.
- `ErrorCode` gains `account-missing`, `account-default-missing` and `account-default-several`, read off pimalaya-config's `AccountError` in the chain by `error::code`, so every failure object (command line, REPL, MCP tool results) carries them; the MCP server failing to resolve its account still reports as text on stderr.
- The `ortie-error` schema lists them; tests/codes.rs checks each through the binary under `--json`.
- Spec (config, output), CHANGELOG and MIGRATION.md (From v2 to v3) follow.
