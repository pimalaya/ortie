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
All endpoints (`endpoints.authorization`, `endpoints.token`, `endpoints.revocation`, `endpoints.redirection`) SHALL be optional at parse time. Each command checks only the endpoints it needs and fails with an error naming the missing field: `token show` needs none until it refreshes, `token refresh` needs `token`, `token revoke` needs `revocation`, `auth get` needs the configured grant's endpoints.

### Requirement: PKCE config shape
The `pkce` field SHALL accept a bool-or-string value: `true` and `"s256"` mean S256, `"plain"` is the escape hatch for broken servers, `false` opts out. The default when omitted is S256. The field applies to the authorization code grant only and is ignored by grants without PKCE.

### Requirement: Extras passthrough
An account MAY carry a raw `[accounts.<name>.extras]` table whose keys are wire parameter names (never kebab-renamed) and whose values are strings. Extras are forwarded verbatim into the configured grant's initiation request (the authorization URL query for the authorization code grant). This carries provider options such as Google `access_type = "offline"` without Ortie learning provider-specific logic; an extra named `resource` replaces the `resources` list on the authorization URL.

### Requirement: Resource indicators
An account MAY carry `resources`, a list of RFC 8707 resource indicator URLs, sent as one `resource` parameter each on the authorization request and on the code exchange.

### Requirement: Storage commands
An account SHALL define read and write storage as external shell commands, each keyed `command`, `cmd` its alias, as a shell line or a program and its arguments (pimalaya-config's `CommandConfig`). The read command prints the token response JSON on stdout; the write command receives it on stdin. Ortie never persists tokens itself.

A storage command exiting non-zero SHALL be reported by its trimmed stderr, or by its program and exit status when it wrote none, never by its stdout, which holds the token or echoes it: in the failure object, the `storage` row of `account check` and the MCP tools alike. A failing read command's report SHALL end with ", run `<command>` yourself to see why", the command written as configured.

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, the `hooks` table and the `tls` string aside, refused as any key it does not read or value of the wrong type ("The configuration refuses every key it does not read"): `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The behaviours an existing account can notice are PKCE by default and `token show` refreshing an expired token, unless it sets `auto-refresh = false`.

### Requirement: Account resolution failures name what is missing
An absent or empty `-a` SHALL take the account marked `default = true`; any other name SHALL take the account of that name, `default` included, which is a name like any other.

Each of the four ways account resolution fails SHALL name what is missing or wrong and what to do about it: a missing configuration names the path it looked for and points at config.sample.toml, a missing named account lists the accounts the configuration does hold, sorted, a missing account named `default` adds "omit `-a` to take the account marked default", a missing default names both ways of picking one, and several accounts marked `default = true`, none named, are refused naming them, sorted, rather than one taken.

#### Scenario: Two default accounts
- GIVEN a configuration whose accounts `a` and `b` both set `default = true`
- WHEN `ortie token show` runs without `-a`
- THEN it fails with "Several accounts are marked default (a, b), keep `default = true` on one, or name one with `-a <NAME>`", and `ortie -a b token show` runs on `b`

#### Scenario: `default` is a name
- GIVEN a configuration whose one account `a` sets `default = true`
- WHEN `ortie -a default token show` runs
- THEN it fails with "Account `default` not found, the configuration holds: a, omit `-a` to take the account marked default", coded `account-missing` under `--json`

### Requirement: The configuration path comes from `-c` only
The configuration path SHALL be overridden by `-c` only, repeatable, the first path the base and the next ones merged on top in order. Each occurrence SHALL be one path, never split on a delimiter, and no environment variable SHALL be read for it.

#### Scenario: A Windows path is read whole
- GIVEN `-c C:\Users\me\config.toml`
- WHEN the configuration is loaded
- THEN it is read from that one path, not from `C` and `\Users\me\config.toml`

### Requirement: One TLS conversion, taking the ALPN list
The `tls` table of an account SHALL hold `provider` (`rustls` or `native-tls`) and `rustls.crypto` (`ring` or `aws`), each the first the build carries when omitted, and `cert`, a PEM certificate to trust beside the platform store, and MAY hold `client.cert` and `client.key`, a PEM client certificate presented when the server asks for one (mutual TLS) and its PEM private key (PKCS#8 under native-tls), both required once `client` is given; every path is shell-expanded. These keys are the ones every Pimalaya CLI spells alike (pimalaya-config cairn/spec/toml.md). It SHALL be folded into the runtime TLS handle by a single conversion taking the ALPN list as its argument. No other code path SHALL build that handle from the configuration, so no call site can negotiate an ALPN, or none, by accident.

#### Scenario: Omitted TLS table
- GIVEN an account with no `tls` key
- WHEN a command connects to the token endpoint
- THEN the provider and the crypto are the first the binary was built with, and no extra certificate is trusted

### Requirement: ALPN is a config key
An account MAY declare `alpn`, the list of ALPN identifiers offered during the TLS handshake. It SHALL default to empty, sending no ALPN extension, which is what an OAuth 2.0 endpoint over plain HTTPS expects; a non-empty list overrides it, `["http/1.1"]` being the one meaningful value, for a TLS middlebox refusing a handshake without ALPN. Only rustls reads it, native-tls ignoring ALPN.

### Requirement: A bare invocation shows the help
A bare `ortie` SHALL answer through pimalaya-cli's `clap::meet_bare_invocation`, and so SHALL `--account` with no subcommand, a half-typed command. Under `--json` it SHALL fail coded `usage-invalid`, "A command is required, see --help", exit 2, the configuration left unread. In text it SHALL show the help when a configuration exists, a broken one included, and fail with the missing configuration otherwise, which a person meets as the welcome. Nothing prompts: Ortie writes no configuration.

### Requirement: A missing configuration meets a person with the welcome
When no configuration exists at the path `-c` gave or the default one, and stderr is a terminal with `--json` off, a bare `ortie` and every account-bound command SHALL print the welcome on stderr and exit 1, printing nothing more. The welcome SHALL frame Ortie in one sentence, name the missing path, give the commands creating it from config.sample.toml, invite the reader to edit it and uncomment the options they need, and give the `auth get` command carrying the `-c` paths. When share/doc/ortie/config.sample.toml exists under the prefix holding the binary's directory or under that directory itself, symlinks resolved, the commands SHALL copy it there with a writable mode, after creating the missing directory; otherwise they SHALL download it with `curl --create-dirs`. Every path SHALL be quoted for the platform's shells (single quotes on Unix, double quotes and `curl.exe` on Windows, which cmd and PowerShell both read), so each command runs as printed. Otherwise the missing-configuration error SHALL stand, as JSON under `--json`.

#### Scenario: A first run
- GIVEN no file at $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml or $HOME/.ortierc
- WHEN `ortie` runs in a terminal
- THEN the welcome names $XDG_CONFIG_HOME/ortie/config.toml, the commands creating it from config.sample.toml and `ortie auth get`, and the exit status is 1

#### Scenario: A packaged binary
- GIVEN ortie installed as /usr/bin/ortie with /usr/share/doc/ortie/config.sample.toml, and no configuration
- WHEN `ortie` runs in a terminal
- THEN the welcome creates the configuration directory and copies /usr/share/doc/ortie/config.sample.toml there with mode 644, downloading nothing

### Requirement: One sample configuration, the provider accounts in the README
The repository SHALL ship config.sample.toml, and the release archives and the Nix package SHALL install it as share/doc/ortie/config.sample.toml, which install.sh copies under its prefix. It holds one `[accounts.example]` table with dotted keys and no nested table. It SHALL list the fields every provider shares and each of their values, the default active and the others commented as `#key = value`, a field whose default is to be left out (the `tls` keys) commented whole, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. It SHALL end with a comment pointing to the README for the Google, Microsoft and Fastmail accounts, then a custom section for an application of one's own at any other provider, a paragraph of commented code only headed by its name, holding example values for the client, the endpoints, the redirection and the scopes. The file SHALL hold a working account once the custom section is uncommented and filled in.

The README SHALL carry one subsection per provider under Configuration (Google, Microsoft, Fastmail), each holding its caveats and one fenced toml block per account, a whole `[accounts.<name>]` table to paste: `google`, `microsoft-graph` and `microsoft-imap-smtp` (one account per resource, since Entra issues a token for one), and `fastmail`. Each block SHALL hold the client, the endpoints, the redirection, the resources and the scopes the provider needs, Pimalaya's application where one is released, an empty `client-id` and `client-secret.raw` in the Google one, and storage commands written with `pass` as the example (`pass show ortie/<name>`, `pass insert -m -f ortie/<name>`).

The test suite SHALL parse config.sample.toml through the loader `-c` uses with the custom section uncommented whole, then on top of it each commented element inside it and each line of commented code outside it, uncommented in place of the active lines of its field, and SHALL parse each toml block of the README through the same loader, checking it holds the one account it names and that the README names the four accounts above, so a configuration change that breaks either fails the build.

#### Scenario: A field is renamed
- GIVEN the README's Google block, which sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the block is not updated
- THEN the test suite fails naming the README line of the block and the account `google`

### Requirement: The configuration refuses every key it does not read
The configuration SHALL refuse at load, naming it, every key it does not read, at every level: the document root, which holds `accounts` alone; an account; and every table nested in one (`endpoints`, `tls`, `tls.rustls`, `tls.client`, `storage`, `storage.read`, `storage.write`, `client-secret`). `extras` alone takes any key, its keys being wire parameter names. A key Ortie read in an earlier release SHALL be refused by serde alone, as any key it does not read or value of the wrong type, never by name: the `hooks` table as an unknown field, the `tls` string as a string where a table is expected.

#### Scenario: A typo in a nested table
- GIVEN an account carrying `endpoints.tokn = "https://login.example.com/token"`
- WHEN any command loads the configuration
- THEN it fails naming the unknown field `tokn`

#### Scenario: A 2.x hook
- GIVEN an account carrying `hooks.on-issue.success.command = "notify-send Ortie"`
- WHEN any command loads the configuration
- THEN it fails naming the unknown field `hooks`, pointing at no migration guide

#### Scenario: A 2.x TLS string
- GIVEN an account carrying `tls = "rustls-ring"`
- WHEN any command loads the configuration
- THEN it fails with serde's "invalid type: string", a table being expected
