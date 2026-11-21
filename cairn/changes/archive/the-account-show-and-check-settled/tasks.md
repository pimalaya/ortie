---
cairn: tasks
change: the-account-show-and-check-settled
---

- [x] tests/account.rs: the plain view (defaults, merged layers, redaction and shell quoting, every label and sanitizing), `tls` left out, a `checked` helper asserting one `check-finished` event per row before the report in every check test (11 of 15 failed on aaab579); tests/repl.rs `a_line_takes_the_whole_grammar` with the events (failed on aaab579)
- [x] src/account/show.rs: `fields::Fields` text, `tls` optional, `AccountShowCommandLine` displayed through `CommandConfig`, unit tests of the labels, sanitizing and `tls` (2 failed on aaab579)
- [x] src/account/check.rs: `AccountCheckEvent::CheckFinished`, one per row as it finishes, its text line, a unit test
- [x] src/json_schema.rs: `ortie-account-check` events registered, `tls` no longer required in `AccountShow`, the events test (2 failed on aaab579)
- [x] spec account.md, output.md, repl.md; log
- [x] CHANGELOG.md, MIGRATION.md
