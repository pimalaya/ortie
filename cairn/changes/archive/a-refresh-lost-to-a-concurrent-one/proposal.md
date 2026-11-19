---
cairn: change
id: a-refresh-lost-to-a-concurrent-one
status: landed
created: 2026-11-19
---

# A refresh lost to a concurrent one takes the stored token

## Why

Finding ORT-3 of the CLI family analysis of 2026-11-15 (MEDIUM, correctness, open on 2026-11-16 and 2026-11-19). Nothing coordinates two refreshes of one account, and a REPL or MCP session keeps the token it read first. On a server rotating refresh tokens, every watch of one grant expiring at once (carillon runs `token show` per session open, per reconnect and per push subscription) refreshes with the same refresh token: one wins, the others are refused `invalid_grant` and report the grant as lost although the storage holds a valid token. A session refreshing with the refresh token it read hours earlier meets the same refusal.

## What

Step 1 of the analysis's direction, size S: on `invalid_grant`, read the storage once more; when it holds another refresh token than the one sent, take that token as the outcome, keep it for the session, write nothing. Otherwise the refusal stands.

Not done: step 2, a lock per account, ruled out by the owner on 2026-11-19 unless a sample provider is shown to revoke the whole token family on reuse; and re-reading the storage before every refresh in a session, which would undo the REPL's "the store is unlocked once" for a gain step 1 already covers (a stale session refresh token is answered `invalid_grant`, then re-read).

### Breaking

None: a failure becomes the success it should have been.
