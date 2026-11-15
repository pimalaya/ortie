---
cairn: delta
change: token-revocation
---

## ADDED Requirements

### Requirement: Revoke command
`token revoke` SHALL revoke the stored refresh token when present, else the access token, against `endpoints.revocation` with the matching `token_type_hint` (RFC 7009), and print the kind it revoked (`{revoked}`, `refreshToken` or `accessToken`). A rejection SHALL fail with the server's error code. Storage SHALL be left as it is.

### Requirement: Revocation endpoint prefilled
The wizard SHALL write the server metadata's `revocation_endpoint` as `endpoints.revocation` when it advertises one.

## MODIFIED Requirements

### Requirement: Late-bound endpoints
All endpoints (`endpoints.authorization`, `endpoints.token`, `endpoints.revocation`, `endpoints.redirection`) SHALL be optional at parse time. Each command checks only the endpoints it needs and fails with an error naming the missing field: `token show` needs none, `token refresh` needs `token`, `token revoke` needs `revocation`, `auth get` needs the configured grant's endpoints.

## REMOVED Requirements
