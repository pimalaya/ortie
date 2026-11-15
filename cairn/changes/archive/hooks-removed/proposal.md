---
cairn: change
id: hooks-removed
status: landed
created: 2026-11-09
---

# Hooks removed

## Why

Ortie is moving to a raw CLI: plain text and JSON out, the rest left to the caller. dependencies-trimmed (2026-11-09) already took the hooks' `notify` tables out for that reason, leaving each hook a command fed the outcome as environment variables. The owner asked on 2026-11-09 to take the hooks out too.

Nothing is lost that the CLI does not already give: `storage.write.command` receives the token JSON on every issue and refresh, so a command chained after the store does what a success hook did, and a failure shows in the exit code and the error Ortie prints, as JSON under `--json`. The one case a hook covered alone, a refresh failing inside a program that swallows `token show`'s output, is covered by checking that program's exit code.

## What

- **BREAKING**: `hooks.on-issue` and `hooks.on-refresh` leave the configuration; a `hooks` table is refused like any unknown key.
- `HooksConfig`, `HookStatusConfig`, `HookConfig` (src/config.rs), the hook commands and runners of `Account` (src/account.rs), their calls in `auth get`, `auth resume`, `token refresh` and the MCP server, and `device_poll_client_error_hook_params` (src/auth/get.rs), which only fed the on-issue error hook, go.
- No dependency leaves: the hooks ran on std's `Command`, and `log` and `secrecy` stay in use elsewhere.
- hook-variables-camel-case, which renamed the hook variables at 3.0, is abandoned.
- config.sample.toml, README, CONTRIBUTING, the src/main.rs header and the CHANGELOG drop the hooks.
