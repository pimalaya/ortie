---
cairn: change
id: the-provider-accounts-in-the-readme
status: landed
created: 2026-11-21
---

# The provider accounts in the README

## Why

Pimalaya guideline readme-011 now puts each named provider's ready-made configuration block in its README subsection, as a fenced toml snippet, config.sample.toml keeping only the annotated field skeleton and pointing to the README. Ortie's README told the reader to uncomment a sample section instead, and its Alternatives section, which readme-007 allows again between Usage and License, was gone.

## What

- README.md: the Google, Microsoft and Fastmail subsections each carry their caveats and one toml block per account (`google`, `microsoft-graph`, `microsoft-imap-smtp`, `fastmail`), a whole `[accounts.<name>]` table with the keys of the former sample sections and `pass` storage commands. The Configuration TIP and the Features bullet follow.
- README.md: `## Alternatives` restored (pizauth, oama, mutt_oauth2.py, each with how it differs), its table of contents entry before AI policy.
- config.sample.toml: the Google, Microsoft Graph, Microsoft IMAP and SMTP, and Fastmail sections dropped for a comment pointing to the README; the skeleton and the custom section kept.
- src/config.rs: the sample test checks the custom section only; a new test parses every toml block of the README through the loader `-c` uses and checks the accounts it names.
- tests/common.rs: `readme_account` replaces `sample_section`, read by the live suites.
- MIGRATION.md, src/main.rs and CHANGELOG.md name the README accounts where they named the sample sections.
- Kept: the welcome and tests/first_run.rs pointing to config.sample.toml, still shipped.
