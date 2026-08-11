---
cairn: spec
capability: config
status: current
---

# Config

Ortie is configured through TOML, one table per account under `[accounts.<name>]`. The config layer holds pure DTOs (`*Config` types) that mirror the nested TOML shape and carry no behaviour; the selected account is flattened into a runtime `Account` view that commands consume. Config files stay entirely user-owned: Ortie never writes them.

### Requirement: Flat grant selector
An account SHALL declare its OAuth 2.0 grant as a flat `grant` field, one of `authorization-code` (the default), `device`, `client-credentials` or `client-credentials-jwt`. `auth get` runs whatever grant the account declares; there is no `--grant` CLI flag and the grant is never inferred from which endpoints are present.

#### Scenario: Omitted grant
- GIVEN an account with no `grant` field
- WHEN a command resolves the account
- THEN the grant is `authorization-code`

### Requirement: JWT assertion credentials
A `client-credentials-jwt` account SHALL declare `client-key`, the path to a PKCS#8 or PKCS#1 PEM private key, and MAY declare `client-certificate`, the path to a PEM or DER certificate whose SHA-1 thumbprint rides as the assertion `x5t` header (required by Microsoft). Every mint SHALL produce a fresh assertion: key re-read from disk, `x5t` recomputed, new `iat` and `exp` on a short validity, unique `jti`, `iss` and `sub` both the client id, `aud` the token endpoint. Assertions are never stored, and `client_secret` never rides along with an assertion.

### Requirement: Late-bound endpoints
All endpoints (`endpoints.authorization`, `endpoints.token`, `endpoints.redirection`) SHALL be optional at parse time. Each command checks only the endpoints it needs and fails with an error naming the missing field: `token show` needs none, `token refresh` needs `token`, `auth get` needs the configured grant's endpoints.

### Requirement: PKCE config shape
The `pkce` field SHALL accept a bool-or-string value: `true` and `"s256"` mean S256, `"plain"` is the escape hatch for broken servers, `false` opts out. The default when omitted is S256. The field applies to the authorization code grant only and is ignored by grants without PKCE.

### Requirement: Extras passthrough
An account MAY carry a raw `[accounts.<name>.extras]` table whose keys are wire parameter names (never kebab-renamed) and whose values are strings. Extras are forwarded verbatim into the configured grant's initiation request (the authorization URL query for the authorization code grant). This carries provider options such as Google `access_type = "offline"` and the RFC 8707 `resource` without Ortie learning provider-specific logic.

### Requirement: Storage commands
An account SHALL define read and write storage as external shell commands. The read command prints the token response JSON on stdout; the write command receives it on stdin. Ortie never persists tokens itself.

### Requirement: Hooks
An account MAY define hooks fired on token issuance and refresh, each with success and error variants. A hook MAY run a command (with the token or error exposed as environment variables) and, under the `notify` feature, show a desktop notification. Secrets travel as secret strings and are never logged.

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged: `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The only behaviour an existing account can notice is PKCE-by-default.
