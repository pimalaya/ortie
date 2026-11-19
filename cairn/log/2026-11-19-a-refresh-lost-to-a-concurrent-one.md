---
cairn: log
change: a-refresh-lost-to-a-concurrent-one
landed: 2026-11-19
---

# A refresh lost to a concurrent one takes the stored token

Finding ORT-3 of the CLI family analysis of 2026-11-15, step 1. src/token/refresh.rs: a refresh refused `invalid_grant` reads the storage once more and, when it holds another refresh token than the one sent, returns that token and keeps it in the session, writing nothing; otherwise the refusal stands, coded `grant-invalid`. It covers both cases the finding names: a concurrent refresh by another process of the same account, and a REPL or MCP session refreshing with the token it read first.

Evidence: tests/token_refresh.rs `a_refresh_another_process_won_takes_the_token_it_stored` (a mock token endpoint storing a rotated token before answering `invalid_grant`, for `token show` and `token refresh`) failed on 58de3a9 with `grant-invalid`; `a_refused_refresh_with_an_unchanged_store_stays_refused` guards the refusal and passes on both.

Ruled by the owner on 2026-11-19: no lock per account (step 2) unless a sample provider is shown to revoke the whole token family when a superseded refresh token is reused. The rotation policy of the three sample providers is not established, and a lock would cover neither a second machine nor another client of the grant.

Capabilities moved: token ("Refresh rotation ordering"), mcp ("An MCP server over stdio").
