---
cairn: tasks
change: the-contract-audit-rulings
---

- [x] src/config.rs: `HooksRemoved` and `tls_table` removed, `TlsClientConfig` added, their unit tests
- [x] src/account/check.rs: the list of accounts, `tls` on the endpoints row, tests/account.rs and tests/repl.rs
- [x] src/auth.rs, src/auth/listen.rs: refusals through `usage_error`, tests/auth_listen.rs
- [x] src/auth/handoff.rs, src/error.rs: `check_stdin` and `stdin-taken`, tests/auth_listen.rs
- [x] src/token.rs, src/auth.rs: hidden aliases
- [x] src/main.rs, src/cli.rs: the kit's bare invocation, tests/first_run.rs
- [x] config.sample.toml: `tls.client`
- [x] spec config.md, account.md, auth.md, output.md; MIGRATION.md, CHANGELOG, log
