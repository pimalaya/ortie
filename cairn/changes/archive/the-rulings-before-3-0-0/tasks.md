---
cairn: tasks
change: the-rulings-before-3-0-0
---

- [x] src/token/issued.rs: `account` first in `TokenIssuedOutput`, the text naming it; src/token/refresh.rs and src/auth/get.rs pass the account name
- [x] src/auth/get.rs: `interactive` skipped from the serialization and the schema of `AuthorizationUri` and `DeviceAuthorization`
- [x] src/error.rs: `redirection-unsupported`; src/auth/listen.rs: the configured redirection's refusals exit 1 coded, the `--redirect-uri` one stays exit 2
- [x] `///` help of `auth get`, `auth listen`, `auth resume` and `token refresh`: output and exit codes
- [x] `client-cert` in src/config.rs tests (the alias checked), src/main.rs, tests/client_credentials.rs, tests/microsoft.rs, cairn/spec/config.md
- [x] tests: auth_listen.rs (receipt `account` first, text receipt, no `interactive`, an older handoff with `interactive` read, `a_redirection_the_config_lacks_or_cannot_listen_on_exits_1_coded`, the https config refusal at exit 1), device_grant.rs (no `interactive`), token_refresh.rs and mcp.rs (receipt `account`), client_credentials.rs (text receipt); each failed on 0fc7fb2
- [x] spec output.md, auth.md, config.md; log
- [x] CHANGELOG.md and MIGRATION.md lines
