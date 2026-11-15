---
cairn: log
change: older-migration-guides
landed: 2026-11-15
---

# Older migration guides

MIGRATION.md gains "From v1 to v2" and "From v0.1 to v1" after "From v2 to v3", per the owner rule of 2026-11-15 (one section per major from v0.1 on; the tags are v0.1.0, v1.0.0, v1.1.0 and v2.0.0 to v2.3.0). Each was built from the CHANGELOG and the tagged sources (`git show <tag>:<path>`), with the pimalaya-toolbox 0.0.4, io-process 0.0.2 and io-oauth 0.0.4 and 0.2.1 crate sources v1 and v2 built on, and says so.

One finding the CHANGELOG never named: v1 stored the token with an `issued_at` object (serde's `SystemTime`), which v2 and v3 read as epoch seconds, so a v1-stored token fails to parse after the upgrade. Reproduced with the v3 debug binary on a scratch token ("invalid type: map, expected u64"); the guide says to run `auth get` once, or drop the field with jq then `token refresh`.

Docs only, no proposal folder; no spec moved.
