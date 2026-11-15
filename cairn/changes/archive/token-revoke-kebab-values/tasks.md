---
cairn: tasks
change: token-revoke-kebab-values
---

# Tasks

- [x] src/token/revoke.rs: `TokenKind` serialized `rename_all = "kebab-case"`
- [x] src/mcp.rs: `McpAuthGetKind` and `McpAuthStatusKind` serialized `rename_all = "kebab-case"`, no value changing
- [x] Tests: tests/token_revoke.rs asserts `refresh-token` and `access-token`, both failing on the old code (`refreshToken`, `accessToken`)
- [x] Spec: token ("Revoke command"), output ("Shared shapes and spellings")
- [x] CHANGELOG (the `token revoke` line under Added), MIGRATION.md (From v2 to v3, New commands)
