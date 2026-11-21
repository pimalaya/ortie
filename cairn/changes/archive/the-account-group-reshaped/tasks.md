---
cairn: tasks
change: the-account-group-reshaped
---

- [x] tests/account.rs: the new list, check and show shapes (13 of 15 failed on 020a0e8, the two passing being unchanged behaviour), tests/repl.rs `a_line_takes_the_whole_grammar` (failed on 020a0e8)
- [x] src/error.rs: `token-expired`
- [x] src/config.rs: `GrantConfig`, `TlsProviderConfig` and `RustlsCryptoConfig` serialize as spelled, `GrantConfig::as_str`
- [x] src/account/list.rs: `AccountListOutput`, `AccountListRow`
- [x] src/account/check.rs: `AccountCheckOutput`, `AccountCheckRow`, `AccountCheckTls`, the token row, one row per endpoint
- [x] src/account/show.rs: `AccountShowCommand`, `AccountShowOutput`, the TOML text, the round-trip and redaction unit tests
- [x] src/json_schema.rs: the three account schemas, `ortie-account-show`, the required-keys test
- [x] spec account.md, output.md; log
- [x] CHANGELOG.md, MIGRATION.md, README.md usage
