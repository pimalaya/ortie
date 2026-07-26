---
cairn: change
id: tag-inline-comments
status: landed
created: 2026-07-26
---

# Tag inline comments with NOTE

## Why
A conformance pass against .github/GUIDELINES.md (inline-004) found bare inline `//` comments across the auth and repl commands. The rule wants such comments either removed or prefixed with one of NOTE, TODO, FIXME, HACK, SAFETY. Ortie was otherwise clean: module headers, comment widths, manifest, README, CHANGELOG and links all pass.

## What
Prefix every remaining bare explanatory comment with NOTE (they are all non-obvious facts a reader needs: spec quirks, provider edge cases, protocol rationale) in repl.rs, auth/discover.rs, auth/get.rs and auth/resume.rs. Reflow the one comment that then crossed 80 columns. Documentation only, no behaviour change.
