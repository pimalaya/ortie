---
cairn: tasks
change: token-inspect-expires-at
---

# Tasks

- [x] src/token/inspect.rs: `TokenInspectOutput::expires_at` from `issued_at` and `expires_in`, serialized as an RFC 3339 instant clamped to the year 9999; the text's `Expires` field read from it
- [x] Tests: tests/token_inspect.rs (a token issued at 1000 reads `expired` and `expiresAt` 1970-01-01T01:16:40Z, a live one `in 49m` and its instant, one without `issued_at` `unknown` and `null`), all three failing on the old code; the unit test `a_lifetime_past_the_year_9999_expires_at_its_last_second`, panicking without the clamp
- [x] Spec: token ("Token inspect reports metadata, not secrets"), output ("Shared shapes and spellings")
- [x] CHANGELOG (the `token inspect --json` line), MIGRATION.md (From v2 to v3, Output)
