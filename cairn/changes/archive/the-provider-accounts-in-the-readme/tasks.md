---
cairn: tasks
change: the-provider-accounts-in-the-readme
---

- [x] README.md: one toml block per provider account, the TIP and the Features bullet updated
- [x] README.md: `## Alternatives` and its table of contents entry restored
- [x] config.sample.toml: provider sections replaced by a pointer to the README
- [x] src/config.rs: `SECTIONS` down to the custom one, `the_readme_provider_accounts_parse` added and shown failing on a misnamed key
- [x] tests/common.rs: `readme_account` replaces `sample_section`; tests/google.rs, tests/microsoft.rs, tests/fastmail.rs follow
- [x] src/main.rs: the architecture header names the README accounts
- [x] Spec folded: config, packaging, provider-quirks
- [x] CHANGELOG.md entry rewritten
- [x] Log written
