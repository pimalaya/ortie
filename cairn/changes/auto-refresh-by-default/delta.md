---
cairn: delta
change: auto-refresh-by-default
---

## MODIFIED Requirements

### Requirement: Auto-refresh on show
When auto-refresh is enabled (per-command `--auto-refresh`, or the account's `auto-refresh` setting, on unless set to false) and the stored token is expired and a refresh token is present, `token show` SHALL refresh before printing.

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, the `hooks` table aside, refused like any unknown key: `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The behaviours an existing account can notice are PKCE by default and `token show` refreshing an expired token, unless it sets `auto-refresh = false`.
