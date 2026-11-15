---
cairn: log
change: family-order-and-defaults
landed: 2026-11-15
---

# The family's command order and default features

The top-level commands read `auth`, `token`, `repl`, `mcp`, `account`, `completion`, `manual`, `json-schema`. `vendored` joins `rustls-ring` in the default features. pimalaya-cli moves to 0.5.2 and pimalaya-config to 0.5.1 (`cargo update -p`, the Cargo.lock diff kept to their two entries, the resolver's windows-sys flips undone), required as `0.5.2` and `0.5.1` in Cargo.toml.

Evidence: `ortie --help` lists the commands in that order; clippy runs clean with the default features, `--no-default-features --features native-tls` and `--all-features`.

Capabilities moved: packaging ("The command tree follows the family" added, "TLS-only feature set").
