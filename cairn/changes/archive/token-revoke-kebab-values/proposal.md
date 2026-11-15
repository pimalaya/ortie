---
cairn: change
id: token-revoke-kebab-values
status: landed
created: 2026-11-15
---

# `token revoke` prints kebab-case values

## Why

The last check of the CLI alignment round (X3, row M6, schema review A18, 2026-11-15) read `token revoke --json` printing `{"revoked": "refreshToken"}` or `{"revoked": "accessToken"}`. Those values are Ortie's own (src/token/revoke.rs, `TokenKind`, serialized `rename_all = "camelCase"`), and the family contract says "Values a binary defines SHALL be kebab-case; values a protocol defines SHALL keep its spelling" (pimalaya-cli cairn/spec/output.md, "Values and times have one spelling"). camelCase is the rule for keys, not for values.

## What

1. **`revoked` reads `refresh-token` or `access-token`** in `token revoke --json`, the `ortie-token-revoke` schema and the MCP `token_revoke` answer, which runs the same command through the capture printer. The text is unchanged.
2. **Every enum Ortie prints serializes kebab-case**: the MCP `auth_get` `kind` (`authorization`, `device`, `issued`) and `auth_status` `status` (`none`, `pending`, `issued`, `failed`) move from `rename_all = "camelCase"` to `"kebab-case"`. Their variants are one word each, so no value changes; the attribute keeps a later two-word variant from slipping. The error codes and the configuration enums were kebab-case already, no flag or MCP argument takes an Ortie enum, and the remaining values Ortie prints (the token type, the scope, the OAuth codes in a message) are the protocol's.

### Breaking

None against a release: `token revoke` is new in the coming v3 (v2.3.0 has no such command), so no released version printed the camelCase values. The CHANGELOG `token revoke` line under Added and the MIGRATION.md "New commands" line (From v2 to v3) name the values it prints.
