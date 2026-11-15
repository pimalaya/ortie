---
cairn: change
id: token-inspect-expires-at
status: landed
created: 2026-11-15
---

# `token inspect` says when the token expires

## Why

Live testing on 2026-11-15 (CLI alignment line O9) read `token inspect --json` printing `"expiresIn": 3599` beside a text saying `Expires: in 8m 2s`. `expiresIn` was the lifetime the server granted at issue, copied from storage whatever time had passed since; the text adds it to the stored issue time and subtracts the wall clock. The requirement "Token inspect reports metadata, not secrets" asks for what remains of the lifetime in both renderings: the JSON did not say it.

## What

1. **`expiresAt` replaces `expiresIn`** in `token inspect --json`, the `ortie-token-inspect` schema and the MCP `token_inspect` answer: the issue time plus the lifetime, an RFC 3339 instant in UTC, in the past once the token expired, `null` when the store holds no issue time or no lifetime.
2. **The text reads the same value**: `in <duration>` while it lies ahead, `expired` once passed, `unknown` when it is `null`. A token stored without its issue time read `in <lifetime>`, as if issued at that instant; it now reads `unknown`, as the JSON does. Ortie stamps `issued_at` on every token it stores (token spec, "Local issuance timestamp"), so only a token written by hand or by another tool lacks it.
3. **An instant past the year 9999 is clamped** to 9999-12-31T23:59:59Z, the last second RFC 3339 writes: a server stating a lifetime near `u64::MAX` made the addition to `SystemTime` panic.

### The shape chosen

The two shapes put on the table were `expiresIn` as the remaining seconds (same key, new meaning) and an `expiresAt` instant, keeping or dropping `expiresIn`. `expiresAt`, dropping `expiresIn`, is chosen:

- **What is stored is an issue time and a lifetime**, so their sum is a fact of the stored token, the same at every run, while the remaining seconds are that fact minus the clock of the run. A script comparing two runs, a cached output, or an MCP answer an agent keeps in context stays right with the instant and goes stale with a countdown.
- **It pairs with `issuedAt`**: the JSON prints both moments as instants, the text renders both relative to now (`51m 57s ago`, `in 8m 2s`). One rule for the two fields.
- **An expired token needs no convention**: its instant lies in the past. Remaining seconds would clamp at 0, losing how long ago it expired, or go negative, a duration that is not one.
- **The family allows both** (cli/cairn/spec/output.md, "Values and times have one spelling": an instant RFC 3339 in UTC with `Z`, a duration integer seconds) and Ortie already prints `issuedAt` as the first.
- **`expiresIn` keeps one meaning in Ortie**: the lifetime the server stated, which the receipts of `token refresh`, `auth resume` and `auth get` print at issue, when it is also what remains. Giving it the remaining seconds on `inspect` would split that meaning in one binary, and a script written against the old value would read a different number without any error. Keeping it beside `expiresAt` keeps the key live testing misread, for a value that is `expiresAt` minus `issuedAt` whenever both are known; dropping it leaves the JSON one key per text field.

comodoro prints a running timer's remaining seconds (`cycle.duration`), but a timer is a countdown its client polls; a stored token is a record.

### Breaking

`expiresIn` leaves `token inspect --json`; v2 printed `expires_in`, the lifetime. MIGRATION.md (From v2 to v3, Output) gives before and after and the jq lines for the seconds left and the lifetime.
