---
cairn: log
change: tag-inline-comments
landed: 2026-07-26
---

# Tag inline comments with NOTE

Ran the Pimalaya guidelines audit on ortie the same way as sirup. Ortie already passed on module headers, comment widths, dashed banners, the manifest, the README, the CHANGELOG and its links. The one gap was inline-004: fifteen bare explanatory `//` comments across repl.rs, auth/discover.rs, auth/get.rs and auth/resume.rs. Prefixed each with NOTE (all are non-obvious facts: spec quirks, Fastmail/provider edge cases, protocol rationale) and reflowed the one comment that then crossed 80 columns.

Documentation only, no behaviour change, so the spec did not move. build, clippy and fmt are clean.
