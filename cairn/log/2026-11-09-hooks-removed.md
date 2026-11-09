---
cairn: log
change: hooks-removed
landed: 2026-11-09
---

# Hooks removed

The hooks are gone. `AccountConfig` lost `hooks`, and `HooksConfig`, `HookStatusConfig`, `HookConfig` and `deserialize_opt_command` left src/config.rs; `Account` lost its four hook commands, `execute_on_{issue,refresh}_{success,error}_hook` and the runners that exported `ACCESS_TOKEN`, `TOKEN_TYPE`, `EXPIRES_IN`, `REFRESH_TOKEN`, `SCOPE`, `ERROR`, `ERROR_DESCRIPTION` and `ERROR_URI`. `auth get`, `auth resume`, `token refresh` and the MCP server only store the token or report the error now, and `device_poll_client_error_hook_params` (src/auth/get.rs), which synthesised an `expired_token` for the on-issue error hook, left with its two tests; an expired device code still fails with `DeviceCodeExpired`. A `hooks` table is refused as an unknown field, which `a_hooks_table_is_refused` checks (a hook command parsed before).

Cargo.toml and Cargo.lock are unchanged: the hooks ran on std's `Command`, and `log` and `secrecy` stay in use. The crates went with the `notify` tables in dependencies-trimmed.

hook-variables-camel-case is abandoned. config.sample.toml lost its hooks block, the README its Hooks feature line (storage commands chained after the store stand in), CONTRIBUTING and the src/main.rs header their mention, and the CHANGELOG's `notify` removal became the hooks' removal.

Capabilities moved: config (cairn/spec/config.md: "Hooks" removed, "v1 config compatibility" modified), auth, mcp and token (cairn/spec/auth.md, mcp.md, token.md: hook clauses dropped).

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`.
