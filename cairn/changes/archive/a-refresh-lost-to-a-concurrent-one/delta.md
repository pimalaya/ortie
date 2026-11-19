---
cairn: delta
change: a-refresh-lost-to-a-concurrent-one
---

## MODIFIED Requirements
### Requirement: Refresh rotation ordering
When the server refuses the refresh token as `invalid_grant`, Ortie SHALL read the storage once more and, when it now holds another refresh token, take that token as the outcome, without writing it back, and keep it for the session. When the storage holds the refused refresh token still, or cannot be read, the refusal stands, coded `grant-invalid`. Nothing else coordinates two refreshes of one account: Ortie SHALL take no lock across processes.

### Requirement: An MCP server over stdio
The stored token SHALL be read at most once for the server's lifetime, as in the REPL, save the one re-read a refused refresh makes.
