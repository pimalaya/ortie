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
`token show` SHALL treat a token as expired when `issued_at + expires_in` is within a fixed skew (60 seconds) of the wall clock. When `expires_in` is absent it SHALL default to one hour (3600 seconds). When `issued_at` is absent the token is assumed still valid.

### Requirement: Local issuance timestamp
On every successful issuance or refresh, Ortie SHALL stamp `issued_at` with its own wall clock at receipt on the token it persists and caches, overriding any server `Date` header, so expiry is computed against a single clock. On refresh the new token SHALL be persisted to storage before the in-memory token is replaced.

### Requirement: Auto-refresh on show
When auto-refresh is enabled (per-command `--auto-refresh` or the account's `auto-refresh` setting) and the stored token is expired and a refresh token is present, `token show` SHALL refresh before printing.

### Requirement: Refresh rotation ordering
On refresh, Ortie SHALL persist the new token to storage before the old one can be lost, and SHALL keep the previous refresh token when the server omits a rotated one. A refresh failure fires the on-refresh error hook and reports the server error code.

### Requirement: Auto-refresh branches per grant
When auto-refresh triggers, Ortie SHALL exchange the refresh token where one exists and SHALL silently re-acquire (re-run the configured grant) on the client credentials kinds, which issue no refresh token. `token refresh` SHALL follow the same decision. On an auto-refreshing client credentials account, a stored token that is missing or unreadable SHALL re-acquire instead of failing, so `token show --auto-refresh` transparently produces a valid token for every grant.
