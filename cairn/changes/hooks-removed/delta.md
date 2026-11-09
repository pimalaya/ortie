---
cairn: delta
change: hooks-removed
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, the `hooks` table aside, refused like any unknown key: `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The only behaviour an existing account can notice is PKCE-by-default.

In auth.md, mcp.md and token.md, the requirements on the authorization code, device and client credentials grants, the MCP loopback flow and refresh lose their "fire the on-issue hooks" and "fires the on-refresh error hook" clauses, unchanged otherwise.

## REMOVED Requirements

### Requirement: Hooks
Removed: `storage.write.command` already receives every issued and refreshed token, and a failure shows in the exit code and the printed error, so the hooks duplicated what the CLI gives.
