---
cairn: log
change: migration-guide
landed: 2026-11-15
---

# Migration guide

Ortie follows the owner's rule of 2026-11-15: every major version from v0.1 on gets a migration guide. MIGRATION.md holds "From v2 to v3", one item per breaking line of the CHANGELOG [Unreleased] (and the behaviour changes a script notices: no browser opened, `--pkce` validation, REPL session flags, OAuth refusal spelling, text layouts), each with what to change in the configuration, scripts and JSON consumers, before/after snippets and jq recipes. The CHANGELOG [Unreleased] opens with a line pointing at it, as comodoro's does.

The `hooks` refusal (src/config.rs `HooksRemovedConfig`) now links https://github.com/pimalaya/ortie/blob/master/MIGRATION.md instead of the CHANGELOG, and `a_removed_key_points_at_the_migration_notes` asserts `MIGRATION.md`; the CHANGELOG hooks entry says the same. Trivial change, no proposal folder.

Seen with the debug binary on a scratch home: the `hooks` refusal naming MIGRATION.md, two `default = true` accounts refused naming them, an empty storage coded `token-missing`, and `--json token show --nope` printing the `usage-invalid` object and exiting 2.

Capabilities moved: config (cairn/spec/config.md: "The configuration refuses every key it does not read" links MIGRATION.md).
