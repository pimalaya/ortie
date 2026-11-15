---
cairn: log
change: family-housekeeping
landed: 2026-11-15
---

# Family housekeeping

Lines X2 and X6 of the family's CLI alignment round (pimalaya/.github cairn/changes/cli-alignment-2026-11-15).

Every closed change moved from cairn/changes/ to cairn/changes/archive/, as in cardamum's cairn: the 54 landed ones, token-inspect-expires-at among them, and the abandoned hook-variables-camel-case, so changes/ holds in-flight proposals only. The two proposals naming another change by path, auth-listen and mcp-server, point into the archive. cairn/verify.sh is calendula's, unchanged, and reports the cairn conformant; CONTRIBUTING.md names it.

.github/workflows/tests.yml gains the `features` job in cardamum's shape: clippy with `-D warnings` over the default features, `--all-features` and each TLS provider alone (rustls-ring, rustls-aws, native-tls).

pimalaya-config moves to 0.5.2, required by both dependency tables of Cargo.toml, the Cargo.lock diff that crate alone: an unknown key's error no longer lists `hooks`, which the loader refuses as removed, and keeps `tls`, read as a table. Seen on the debug binary with an unknown account key.

The failure code table stays in cairn/spec/output.md: Ortie has no commands spec, and output.md holds its exit statuses and the failure object. MIGRATION.md already reads "From v2 to v3", "From v1 to v2" and "From v0.1 to v1", and tests/ already holds exit.rs, first_run.rs and account.rs.

Docs and build only, no proposal folder; no spec moved.
