---
cairn: change
id: tls-table
status: landed
created: 2026-11-15
---

# The `tls` table, and the configuration on the family's shapes

## Why

Every other Pimalaya CLI reads TLS as a table, `tls.provider`, `tls.rustls.crypto` and `tls.cert` (himalaya/src/config/tls.rs, cardamum, calendula); Ortie read one string, `tls = "auto|native-tls|rustls-aws|rustls-ring"` (finding F4, owner ruling D4: "tls key should be the same everywhere"). Its removed-key refusal had its own sentence where pimalaya-config 0.5.1 gives the family one (F41), and its storage commands went through the `Command` adapter where pimalaya-config ships `CommandConfig`, the shape the secrets keep (F44, D6).

## What

1. **`tls` is a table**: `tls.provider` (`rustls`, `native-tls`) and `tls.rustls.crypto` (`ring`, `aws`), each the first the build carries when omitted, as `tls = "auto"` was, and `tls.cert`, a PEM certificate to trust beside the platform store, path shell-expanded, new to Ortie. The table refuses an unknown key. `tls.client` (himalaya's mutual TLS) is not taken: no sibling but himalaya has it.
2. **The 2.x string is refused by name**, in the family sentence, its replacement given per value: `The \`tls\` key is removed, the TLS options being a table as in every Pimalaya CLI: write \`tls.provider = "rustls"\` and \`tls.rustls.crypto = "ring"\`, see MIGRATION.md` for `rustls-ring`, `delete it, the build's provider being the default` for `auto`. It goes through `pimalaya_config::toml::Removed`, the type `removed_key!` wraps, since `removed_key!` builds a type refusing every value of a key while `tls` keeps its table, and the sentence names the keys of the value read.
3. **The `hooks` table is refused through `removed_key!`**: "The `hooks` table is removed, the storage write command running on every issue and refresh: chain a command after `storage.write.command`, see MIGRATION.md". It no longer links MIGRATION.md by URL, as no sibling does.
4. **Storage commands on `CommandConfig`**, built into a `Command` when one runs; `cmd` stays the alias of `command`. Nothing changes for a configuration.

This renames a configuration key: an account setting `tls` fails to load until it moves to the table. The live tester's configuration sets none (2026-11-15).
