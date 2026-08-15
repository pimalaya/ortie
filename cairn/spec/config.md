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

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: The offer is a hook, not a gate
A missing configuration SHALL raise an offer to generate one, from a bare invocation and from any command needing an account. The offer never ends the process: a command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail on the configuration it still has not got. A bare invocation has nothing to carry on to, so a declined offer falls back to the help. Nothing is offered when stdin is not a terminal or `--json` is set.

### Requirement: A bare invocation meets the newcomer, not the user
A bare `ortie` SHALL raise the offer when it finds no configuration, and show the help when it finds one, since someone already set up is asking what the commands are. `--account` with no subcommand is a half-typed command and shows the help too.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `ORTIE_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run. It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: A generated account takes a free name and one default
The account name SHALL be suffixed until the configuration does not already hold it, since a second `[accounts.<name>]` table makes the whole document fail to parse. The generated account claims `default` only when no other account does, since two defaults resolve to whichever the account map yields first.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: The configuration path is read from the environment
The configuration path SHALL be read from `ORTIE_CONFIG` as well as `-c`, both accepting a `:`-delimited list merged in order.

### Requirement: The wizard saves where the configuration lives
The save SHALL NOT prompt for a path: it writes where `-c` or `ORTIE_CONFIG` pointed, or the default location. A file already holding accounts is still appended to rather than overwritten, and still confirmed before it happens, since it is one the user already owns. The fragment still reaches stdout before the save is offered, so the choice is made having seen what is being placed.
