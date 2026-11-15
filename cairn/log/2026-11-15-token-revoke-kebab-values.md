---
cairn: log
change: token-revoke-kebab-values
landed: 2026-11-15
---

# `token revoke` prints kebab-case values

`token revoke --json` prints `revoked` as `refresh-token` or `access-token`, where it printed `refreshToken` or `accessToken`, as pimalaya-cli's output spec asks of a value a binary defines (CLI alignment X3 row M6, schema review A18). The `ortie-token-revoke` schema and the MCP `token_revoke` answer follow; the text is unchanged. The MCP `auth_get` `kind` and `auth_status` `status` enums serialize kebab-case too, their one-word values unchanged.

Evidence: tests/token_revoke.rs asserts the two new values; both tests fail on the old code, which printed the camelCase ones. `ortie json-schema ortie-token-revoke` lists `access-token` and `refresh-token`.

Breaking: none against a release, `token revoke` being new in v3; the CHANGELOG Added line and MIGRATION.md (From v2 to v3, New commands) name the values.

Capabilities moved: token ("Revoke command"), output ("Shared shapes and spellings").
