---
cairn: log
change: the-rulings-before-3-0-0
landed: 2026-11-21
---

# The rulings before 3.0.0

Four owner decisions of 2026-11-21 on the 3.0.0 contract.

## What reaches users

- The token receipt of `token refresh`, `auth listen`, `auth resume`, a token-issuing `auth get` and the MCP tools reads `{"account", "expiresIn", "withRefreshToken"}`, the account first; its text `Access token of account <name> successfully issued (expires in <duration>)`, or `refreshed`.
- The `auth get` authorization-code and device handoffs no longer print `interactive`; a handoff file an earlier build wrote with it is still read.
- `auth listen` with neither `--redirect-uri` nor `endpoints.redirection` exits 1 coded `endpoint-missing`, and on an `endpoints.redirection` that is not an http loopback URI exits 1 coded `redirection-unsupported` (new), naming `endpoints.redirection`; both exited 2 coded `usage-invalid`. The same fault in `--redirect-uri` or a handoff `redirectUri` stays exit 2 coded `usage-invalid`, as do the other refusals of the-grant-refusals-exit-2, each fixed by changing the command line.
- The configuration key `client-cert` is the one written everywhere; `client-certificate` is still read.

## Capabilities moved

output, auth, config (see the change's delta).

## Verified

`cargo test` (140 passed, 8 ignored); the ten tests carrying the new or changed assertions, in tests/auth_listen.rs (5), tests/client_credentials.rs, tests/device_grant.rs, tests/mcp.rs and tests/token_refresh.rs, failed on 0fc7fb2 before the change. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` under the default features, `native-tls` and `rustls-aws` alone. `json-schema` gains `account` in the receipt schemas, loses `interactive` from `ortie-auth-get` and gains `redirection-unsupported` in `ortie-error`.
