---
cairn: delta
change: headless-grants
---

## ADDED Requirements

### Requirement: Client credentials grants
The flat `grant` selector SHALL accept `client-credentials` (RFC 6749 section 4.4, client authenticated by `client-secret`) and `client-credentials-jwt` (RFC 7523 section 2.2, client authenticated by a signed JWT assertion). On these accounts `auth get` SHALL run the exchange headlessly in one shot against `endpoints.token`, write storage and fire the on-issue hooks; `auth resume` SHALL be rejected since there is nothing to resume.

### Requirement: JWT assertion credentials
A `client-credentials-jwt` account SHALL declare `client-key`, the path to a PKCS#8 or PKCS#1 PEM private key, and MAY declare `client-certificate`, the path to a PEM or DER certificate whose SHA-1 thumbprint rides as the assertion `x5t` header (required by Microsoft). Every mint SHALL produce a fresh assertion: key re-read from disk, `x5t` recomputed, new `iat` and `exp` on a short validity, unique `jti`, `iss` and `sub` both the client id, `aud` the token endpoint. Assertions are never stored, and `client_secret` never rides along with an assertion.

### Requirement: Auto-refresh branches per grant
When auto-refresh triggers, Ortie SHALL exchange the refresh token where one exists and SHALL silently re-acquire (re-run the configured grant) on the client credentials kinds, which issue no refresh token. `token refresh` SHALL follow the same decision. On an auto-refreshing client credentials account, a stored token that is missing or unreadable SHALL re-acquire instead of failing, so `token show --auto-refresh` transparently produces a valid token for every grant.

### Requirement: Certificate renewal hint
When the JWT kind is rejected with `invalid_client`, the reported error SHALL carry a hint that the certificate credential is likely expired and needs renewal.

## MODIFIED Requirements

### Requirement: Flat grant selector
An account SHALL declare its OAuth 2.0 grant as a flat `grant` field, one of `authorization-code` (the default), `device`, `client-credentials` or `client-credentials-jwt`. `auth get` runs whatever grant the account declares; there is no `--grant` CLI flag and the grant is never inferred from which endpoints are present.

#### Scenario: Omitted grant
- GIVEN an account with no `grant` field
- WHEN a command resolves the account
- THEN the grant is `authorization-code`
