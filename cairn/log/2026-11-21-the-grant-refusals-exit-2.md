---
cairn: log
change: the-grant-refusals-exit-2
landed: 2026-11-21
---

# The grant refusals exit 2

The contract check of Ortie before 3.0.0 (every command under `--json`, the REPL and the MCP server against pimalaya-cli 0.8.0 cairn/spec) found every payload shape conforming and one class of exit codes left behind by the 2026-11-20 rulings.

## What reaches users

- `auth listen` on a device or client credentials account, or on a redirection that is not an http URI on a loopback host (from `--redirect-uri` or `endpoints.redirection`), exits 2 coded `usage-invalid`, where it exited 1 uncoded.
- `auth resume` on a client credentials account, with `--state`, `--pkce` or `--redirect-uri` on a device account, with an empty device code, or with a redirected URI that does not parse, exits 2 coded `usage-invalid`, where it exited 1 uncoded.
- The MCP tool `auth_resume` given a URI that does not parse answers its failure coded `usage-invalid`, where `code` was `null`.
- Messages unchanged.

## Capabilities moved

auth, output, mcp (see the change's delta).

## Verified

`cargo test` (125 passed, 8 ignored); the new test `a_command_line_the_grant_refuses_exits_2_coded` and the new `usage-invalid` assertion of `a_mis_pasted_uri_leaves_the_flow_pending` failed on 801e5dd before the change. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` under the default features, `native-tls` and `rustls-aws` alone. The `json-schema` output is unchanged; its `$id` reads v2 until Cargo.toml moves to 3.0.0.
