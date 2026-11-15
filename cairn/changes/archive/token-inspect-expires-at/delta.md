---
cairn: delta
change: token-inspect-expires-at
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Token inspect reports metadata, not secrets
(token) `token inspect` SHALL emit, in both renderings, the metadata of the stored token and not the token itself: the token type, when it was issued, when it expires, whether a refresh token is held, and the granted scope. The expiry SHALL be the stored issue time plus the stored lifetime, unknown when either is absent. The text SHALL render it against the wall clock, `in <duration>` while it lies ahead, `expired` once passed, `unknown` when unknown; the JSON SHALL print it as the instant `expiresAt`, `null` when unknown, an instant past the year 9999 clamped to `9999-12-31T23:59:59Z`. The access token and the refresh token SHALL NOT be serialized. The raw access token stays available from `token show`.

#### Scenario: An expired token
- GIVEN a store holding a token issued at 1000 (Unix seconds) for 3600 seconds
- WHEN `ortie token inspect` runs, then `ortie token inspect --json`
- THEN the text reads `Expires: expired`, and the JSON `"issuedAt": "1970-01-01T00:16:40Z"` and `"expiresAt": "1970-01-01T01:16:40Z"`, with no `expiresIn`

#### Scenario: A token stored without its issue time
- GIVEN a store holding a token with a lifetime and no `issued_at`
- WHEN `ortie token inspect` runs, then `ortie token inspect --json`
- THEN the text reads `Expires: unknown` and the JSON `"expiresAt": null`

### Requirement: Shared shapes and spellings
(output) `token inspect` SHALL print `issuedAt` and `expiresAt` as RFC 3339 instants in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

## REMOVED Requirements

None.
