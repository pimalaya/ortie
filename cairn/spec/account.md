---
cairn: spec
capability: account
status: current
---

# Account

`ortie account` lists the accounts a configuration declares, shows one and checks one, carrying the account group every Pimalaya CLI carries (pimalaya-cli cairn/spec/commands.md) on shapes of its own: Ortie has no backend, and runs on one account. The check stands on what an account needs to work: its token storage, the token it holds, and its endpoints.

### Requirement: The list reads the configuration only
`ortie account list`, alias `ls`, SHALL print every account the configuration declares, sorted by name, as `{"accounts": [{"name", "default", "grant"}]}`: `default` true on the account marked `default = true`, `grant` the grant as the configuration spells it (`authorization-code`, `device`, `client-credentials`, `client-credentials-jwt`), the default one when the account names none. Its text SHALL be one line per account, its name, its grant and `default` on the default one (`-` otherwise), aligned in columns. It SHALL take no account, so it runs whatever `-a` names and however many accounts are marked default, and SHALL reach nothing: no storage command runs and no request is sent.

#### Scenario: Two accounts
- GIVEN a configuration holding `work`, marked default with `grant = "device"`, and `home`, naming no grant
- WHEN `ortie account list --json` runs
- THEN it prints `home` with `authorization-code`, then `work` with `device`, `work` alone with `default` true, and exits 0

### Requirement: The show prints the account as loaded
`ortie account show` SHALL take the account as every account-bound command does and print every property of it as the configuration loads it: every `-c` layer merged (a list of a later layer appended to the earlier one's), every default filled in (`default = false`, `grant = "authorization-code"`, `alpn`, `scopes` and `resources` empty, `pkce = "s256"`, `extras` empty, `auto-refresh = true`), every unset optional left out (`client-secret`, `client-key`, `client-cert`, each endpoint, `tls.provider`, `tls.rustls.crypto`, `tls.cert`, `tls.client`). It SHALL reach nothing: no storage command runs and no request is sent. Its text SHALL be TOML that pastes back into a configuration: `[accounts.<name>]`, a blank line, the keys in their configuration spelling, in this order (`default`, `client-id`, `client-secret`, `client-key`, `client-cert`, `grant`, `endpoints.*`, `tls.*`, `alpn`, `scopes`, `resources`, `pkce`, `extras`, `auto-refresh`), a blank line, then `storage.read.command` and `storage.write.command`; a key or a name that is no bare TOML key quoted, every control and bidi character of a string escaped. Its JSON SHALL be `{"account": {"name", "default", "clientId", "clientSecret", "clientKey", "clientCert", "grant", "endpoints", "tls", "alpn", "scopes", "resources", "pkce", "extras", "autoRefresh", "storage"}}`, the nested keys camelCase too, `extras` keyed by wire name, an unset optional key absent, `pkce` `"s256"`, `"plain"` or `false`. A raw secret (`client-secret.raw`, the one the configuration types hold) SHALL never be printed: the text writes `client-secret.raw = "<redacted>"`, the JSON `{"raw": null, "redacted": true}`. A command (`client-secret.command`, `storage.*.command`) SHALL print as written, a shell line as a string and a program and its arguments as an array; a path (`client-key`, `client-cert`, `tls.cert`, `tls.client.*`) as loaded, shell-expanded.

#### Scenario: The defaults filled in
- GIVEN an account holding only `client-id` and its storage commands
- WHEN `ortie account show --json` runs
- THEN it prints `default` false, `grant` `authorization-code`, `endpoints` `{}`, `tls` `{"rustls": {}}`, empty `alpn`, `scopes`, `resources` and `extras`, `pkce` `s256`, `autoRefresh` true and the storage commands, and no other key

#### Scenario: A raw secret
- GIVEN an account with `client-secret.raw = "s3cret"`
- WHEN `ortie account show` runs, with or without `--json`
- THEN `s3cret` appears nowhere, the text holding `client-secret.raw = "<redacted>"`, and that text with `<redacted>` swapped back for `s3cret` loads the same account

### Requirement: The check proves the storage, the token and the endpoints
`ortie account check` SHALL take the account as every account-bound command does and print `{"account", "checks": [{"check", "ok", "code", "error", "tls"}]}`, its rows `storage`, `token`, then one per endpoint the grant uses, every row checked whatever the others did, each failure coded as the failure object of any other command would be. `tls` SHALL be `null` on `storage`, `token` and every endpoint row but an `https` one, which carries the TLS the account configures, `{provider, crypto, cert, clientCert}`, each `null` where the build's default answers.

- `storage` SHALL run `storage.read.command` once and pass when it exits successfully printing a token JSON that parses, an empty output failing coded `token-missing`; a token a REPL session holds SHALL be left as it is, and `storage.write.command` SHALL NOT run.
- `token` SHALL read the token the storage returned, sending nothing, and pass unless it is expired (by the expiry `token show` uses) with no way to renew it: no refresh token, on a grant other than the client credentials ones, which run again by themselves. Such a token SHALL fail coded `token-expired`. An expired token a refresh token or the grant renews SHALL pass, its `error` `null`. A storage that failed SHALL fail the row: coded `token-missing` with the storage's message on an empty store, uncoded "No access token to check, the storage check failed" otherwise.
- `endpoints.<key>`: the endpoints a grant uses are `endpoints.authorization`, `endpoints.token` and `endpoints.revocation` for `authorization-code`, `endpoints.device-authorization`, `endpoints.token` and `endpoints.revocation` for `device`, `endpoints.token` and `endpoints.revocation` for the client credentials grants, in that order, all but `endpoints.revocation` needed. A needed one the account leaves out SHALL fail its row coded `endpoint-missing`, sending nothing; an unset `endpoints.revocation` SHALL print no row. A configured one SHALL be sent one request over a new connection opened as the grants open theirs: `HEAD <path and query> HTTP/1.1` with the headers `Host` and `Connection: close` and nothing else, no credential, client identifier, cookie or body. Any HTTP status line in answer SHALL pass the row; none, or no HTTP one, SHALL fail it with "Cannot reach endpoints.<key> at <url>: <why>". `endpoints.redirection`, and an endpoint the grant does not use, SHALL be sent nothing.

Its text SHALL be the account then one `<check>: OK` or `<check>: FAIL (<why>)` line per row, aligned as fields, an `https` endpoint row ending in `, TLS` and the configured parts. A failed row SHALL end the command with the partial-outcome status 3 once the report is printed.

#### Scenario: A reachable account
- GIVEN an authorization code account storing an unexpired token, its authorization, token and revocation endpoints on a server answering `405 Method Not Allowed`
- WHEN `ortie account check --json` runs
- THEN the rows `storage`, `token`, `endpoints.authorization`, `endpoints.token` and `endpoints.revocation` pass, the server received three `HEAD` requests carrying only `Host` and `Connection: close`, in that order, and the write command did not run

#### Scenario: An empty store and a closed port
- GIVEN a client credentials account whose store holds nothing and whose token endpoint is a closed port
- WHEN `ortie account check --json` runs
- THEN it prints the rows `storage` and `token` failed coded `token-missing` and `endpoints.token` failed naming it, and exits 3

#### Scenario: An expired token
- GIVEN an authorization code account storing an expired token
- WHEN `ortie account check --json` runs
- THEN the `token` row fails coded `token-expired` when the token holds no refresh token, and passes, `error` `null`, when it holds one

#### Scenario: A needed endpoint left out
- GIVEN a device account configuring `endpoints.authorization` and `endpoints.token` alone
- WHEN `ortie account check --json` runs
- THEN `endpoints.device-authorization` fails coded `endpoint-missing`, `endpoints.token` is sent its `HEAD` and passes, `endpoints.authorization` prints no row and is sent nothing, and it exits 3

#### Scenario: Endpoints over TLS
- GIVEN a client credentials account whose token endpoint is `https://127.0.0.1:1/token` and revocation endpoint `http://127.0.0.1:1/revoke`, with `tls.provider = "rustls"`, `tls.cert` and `tls.client`
- WHEN `ortie account check --json` runs
- THEN the `storage`, `token` and `endpoints.revocation` rows carry `"tls": null` and the `endpoints.token` row `{"provider": "rustls", "crypto": null, "cert": <tls.cert>, "clientCert": <tls.client.cert>}`
