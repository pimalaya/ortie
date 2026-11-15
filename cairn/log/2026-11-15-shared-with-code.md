---
cairn: log
change: shared-with-code
landed: 2026-11-15
---

# The shared `with_code`

pimalaya-cli is pinned at 51ce8b17714ee7445bd59874d0cfd988612d9a08, which ships `pimalaya_cli::error::with_code(err, code)` (owner ruling 2026-11-15, item 13). Ortie's own `error::with_code` is gone; src/main.rs, src/repl.rs, src/cli.rs (the MCP server loop) and src/mcp.rs call the shared one with `error::code`. The private `chained` helper is folded into `oauth_error`, its last caller. The tests of src/error.rs that check Ortie's codes through the report (`a_bare_status_is_coded_with_its_messages_kept`, `another_status_is_left_uncoded`) now call the shared helper.

No behaviour change: the shared helper also returns a chain coded `usage-invalid` or holding a `PartialOutcome` unchanged, and Ortie's `code` finds no code in either (no io-oauth or account error under them), so the old helper returned them unchanged too. The whole suite, tests/codes.rs, tests/exit.rs, tests/repl.rs and tests/mcp.rs included, passes unchanged. Trivial change, no proposal folder; no spec moved.
