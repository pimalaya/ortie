---
cairn: delta
change: token-revoke-kebab-values
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Revoke command
(token) `token revoke` SHALL revoke the stored refresh token when present, else the access token, against `endpoints.revocation` with the matching `token_type_hint` (RFC 7009), and print the kind it revoked (`{revoked}`, `refresh-token` or `access-token`). A rejection SHALL fail with the server's error code. Storage SHALL be left as it is.

### Requirement: Shared shapes and spellings
(output) Values Ortie defines (`revoked`, the MCP `kind` and `status`, the codes) SHALL be kebab-case; values a protocol defines (the token type, the scope, the OAuth codes) SHALL keep its spelling. `token inspect` SHALL print `issuedAt` and `expiresAt` as RFC 3339 instants in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

#### Scenario: The kind of a revoked token
- GIVEN a store holding a refresh token and a revocation endpoint accepting it
- WHEN `ortie token revoke --json` runs
- THEN it prints `{"revoked": "refresh-token"}`

## REMOVED Requirements

None.
