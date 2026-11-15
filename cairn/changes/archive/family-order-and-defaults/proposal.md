---
cairn: change
id: family-order-and-defaults
status: landed
created: 2026-11-15
---

# The family's command order and default features

## Why

The CLI alignment round of 2026-11-15 ruled the standard commands in himalaya's order, `completion`, `manual`, `json-schema`, where Ortie listed `manual` first (finding F27, ruling P5), and `vendored` among the default features, as five of the seven CLIs have it (F43, ruling P1). It also moves Ortie to the libraries the other lines need: pimalaya-cli 0.5.2 (the shared account outputs, the nested-session refusal, the session version) and pimalaya-config 0.5.1 (`removed_key!`, the family sentence).

## What

1. **The top-level commands** read `auth`, `token`, `repl`, `mcp`, `account`, then `completion`, `manual`, `json-schema`; their aliases stay. The abouts of the standard commands are pimalaya-cli's, as in every sibling.
2. **`vendored` is a default feature**: a default build changes nothing on rustls, and a build adding `native-tls` links a vendored OpenSSL, as cardamum's, calendula's and himalaya's do. A packager wanting the system OpenSSL passes `--no-default-features --features native-tls`.
3. **Dependencies**: `cargo update -p pimalaya-cli -p pimalaya-config`, the Cargo.lock diff kept to their two entries; Cargo.toml requires `0.5.2` and `0.5.1`, so a minimal-versions build cannot pick a release lacking what Ortie calls.
