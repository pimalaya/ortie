---
cairn: spec
capability: token
status: current
---

# Token

The `token` command tree works on the access token already persisted in storage: `show` prints it, `inspect` displays its metadata, `refresh` exchanges the refresh token for a fresh one. The token side is grant-agnostic: every grant ends in the same token response shape, so multi-grant support costs nothing downstream of issuance.

### Requirement: Show the raw access token
`token show` SHALL read the token from storage and print the raw access token on stdout, suitable for piping. Under `--json` it prints the token as a JSON object.

### Requirement: Storage round-trip
The token response persisted to and read from storage SHALL be the OAuth 2.0 success-params JSON, carrying at least the access token, token type, optional expiry lifetime, optional refresh token, and the issuance timestamp.

### Requirement: Expiry with skew
`token show` SHALL treat a token as expired when `issued_at + expires_in` is within a fixed skew (60 seconds) of the wall clock, so a token about to lapse is refreshed rather than handed out and rejected mid-request. When either `issued_at` or `expires_in` is unknown, the token is assumed still valid.

### Requirement: Auto-refresh on show
When auto-refresh is enabled (per-command `--auto-refresh` or the account's `auto-refresh` setting) and the stored token is expired and a refresh token is present, `token show` SHALL refresh before printing.

### Requirement: Refresh rotation ordering
On refresh, Ortie SHALL persist the new token to storage before the old one can be lost, and SHALL keep the previous refresh token when the server omits a rotated one. A refresh failure fires the on-refresh error hook and reports the server error code.
