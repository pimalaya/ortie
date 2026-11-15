---
cairn: log
change: token-inspect-expires-at
landed: 2026-11-15
---

# `token inspect` says when the token expires

`token inspect --json` prints `expiresAt`, the stored issue time plus the stored lifetime as an RFC 3339 instant in UTC (`null` when either is absent, clamped to 9999-12-31T23:59:59Z), in place of `expiresIn`, the lifetime granted at issue. The text's `Expires` field reads the same value: `in <duration>`, `expired`, or `unknown`, which a token stored without its issue time now reads where it read `in <lifetime>`. The `ortie-token-inspect` schema and the MCP `token_inspect` answer follow. The proposal holds why the instant was chosen over the remaining seconds.

Evidence: tests/token_inspect.rs runs the binary over a token file; its three tests (issued at 1000 and expired, live, without `issued_at`) fail on the old code, which printed `expiresIn` and read `in 1h` without the issue time. The unit test `a_lifetime_past_the_year_9999_expires_at_its_last_second` panics without the clamp (overflow adding to `SystemTime`).

Breaking: `expiresIn` leaves `token inspect --json`, MIGRATION.md (From v2 to v3, Output).

Capabilities moved: token ("Token inspect reports metadata, not secrets"), output ("Shared shapes and spellings").
