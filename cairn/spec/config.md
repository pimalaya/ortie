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
All endpoints (`endpoints.authorization`, `endpoints.token`, `endpoints.revocation`, `endpoints.redirection`) SHALL be optional at parse time. Each command checks only the endpoints it needs and fails with an error naming the missing field: `token show` needs none, `token refresh` needs `token`, `token revoke` needs `revocation`, `auth get` needs the configured grant's endpoints.

### Requirement: PKCE config shape
The `pkce` field SHALL accept a bool-or-string value: `true` and `"s256"` mean S256, `"plain"` is the escape hatch for broken servers, `false` opts out. The default when omitted is S256. The field applies to the authorization code grant only and is ignored by grants without PKCE.

### Requirement: Extras passthrough
An account MAY carry a raw `[accounts.<name>.extras]` table whose keys are wire parameter names (never kebab-renamed) and whose values are strings. Extras are forwarded verbatim into the configured grant's initiation request (the authorization URL query for the authorization code grant). This carries provider options such as Google `access_type = "offline"` without Ortie learning provider-specific logic; an extra named `resource` replaces the `resources` list on the authorization URL.

### Requirement: Resource indicators
An account MAY carry `resources`, a list of RFC 8707 resource indicator URLs, sent as one `resource` parameter each on the authorization request and on the code exchange.

### Requirement: Storage commands
An account SHALL define read and write storage as external shell commands. The read command prints the token response JSON on stdout; the write command receives it on stdin. Ortie never persists tokens itself.

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, the `hooks` table aside, refused like any unknown key: `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The only behaviour an existing account can notice is PKCE-by-default.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for and points at the samples, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: The configuration path comes from `-c` only
The configuration path SHALL be overridden by `-c` only, repeatable, the first path the base and the next ones merged on top in order. Each occurrence SHALL be one path, never split on a delimiter, and no environment variable SHALL be read for it.

#### Scenario: A Windows path is read whole
- GIVEN `-c C:\Users\me\config.toml`
- WHEN the configuration is loaded
- THEN it is read from that one path, not from `C` and `\Users\me\config.toml`

### Requirement: One TLS conversion, taking the ALPN list
The `tls` account option SHALL name the TLS provider as one of `auto` (the default, the provider the binary was built with), `native-tls`, `rustls-aws` or `rustls-ring`, and SHALL be folded into the runtime TLS handle by a single conversion taking the ALPN list as its argument. No other code path SHALL build that handle from the configuration, so no call site can negotiate an ALPN, or none, by accident.

#### Scenario: Omitted TLS provider
- GIVEN an account with no `tls` field
- WHEN a command connects to the token endpoint
- THEN the provider is the one the binary was built with

### Requirement: ALPN is a config key
An account MAY declare `alpn`, the list of ALPN identifiers offered during the TLS handshake. It SHALL default to empty, sending no ALPN extension, which is what an OAuth 2.0 endpoint over plain HTTPS expects; a non-empty list overrides it, `["http/1.1"]` being the one meaningful value, for a TLS middlebox refusing a handshake without ALPN. Only rustls reads it, native-tls ignoring ALPN.

### Requirement: A bare invocation shows the help
A bare `ortie` SHALL show the help, whether a configuration exists or not, and so SHALL `--account` with no subcommand, a half-typed command. Nothing prompts: Ortie writes no configuration.

### Requirement: Samples, one per provider
The repository SHALL ship config.samples/<provider>.toml, each holding one complete `[accounts.<provider>]` table with dotted keys and no nested table, ready to append to a configuration. Its active lines SHALL use Pimalaya's application where one exists, and the scopes it leaves out SHALL stand as commented code. One comment line SHALL head each scope group, naming what it serves, and no other comment SHALL appear. config.sample.toml SHALL be one `[accounts.example]` table listing every field and each of its values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. The test suite SHALL parse each sample and config.sample.toml through the loader `-c` uses, as it stands and once per line of commented code swapped in for the active line of its key, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose commented code sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

