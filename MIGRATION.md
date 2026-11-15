# Migration guide

## From v2 to v3

v3 brings Ortie onto the standard the Pimalaya CLIs share: no configuration wizard, one configuration path per `-c`, camelCase keys and receipts under `--json`, coded failures, and hooks folded into the storage commands. A v2 configuration loads as it is, unless it holds a `hooks` table, sets `tls`, marks several accounts default, or writes a host in Unicode; a script passing `-a default` drops it.

The [CHANGELOG](./CHANGELOG.md) lists every change; these are the ones to act on.

### Configuration and first run

- **The wizard is gone.** `ortie configure` (alias `wizard`), its `ortie-configure` schema and dynamic client registration (RFC 7591) no longer exist, and bare `ortie` prints the help. Write an account from [config.sample.toml](./config.sample.toml), which releases and packages ship as share/doc/ortie/config.sample.toml: uncomment the section of your provider (Google, Microsoft Graph, Microsoft IMAP and SMTP, Fastmail, or a custom application) and fill its storage commands. Run at a terminal without a configuration, Ortie prints the commands copying it in place. A Fastmail account the wizard registered keeps working; a new one takes the registered client of the sample.
- **The `hooks` table is refused**, by name: "The `hooks` table is removed, the storage write command running on every issue and refresh: chain a command after `storage.write.command`, see MIGRATION.md". `storage.write.command` runs on every token `auth get`, `auth resume`, `auth listen`, `token refresh` and an auto-refreshing `token show` store, so a command chained after it does what a success hook did. The `notify` tables and the `notify` cargo feature went with the hooks: call `notify-send` or any notifier from the command. The variables v2 exported (`ACCESS_TOKEN`, `EXPIRES_IN`, `REFRESH_TOKEN` and so on) are gone; the token JSON on the command's stdin carries the same values, under the OAuth wire names (`access_token`, `expires_in`, `refresh_token`, `scope`).

  ```toml
  # v2
  storage.write.command = "pass insert -m -f ortie/example"
  hooks.on-refresh.success.command = "logger 'ortie refreshed a token'"
  hooks.on-refresh.success.notify.summary = "Ortie"
  hooks.on-refresh.success.notify.body = "Refreshed access token (expires in $EXPIRES_IN)"

  # v3: a plain chain
  storage.write.command = "pass insert -m -f ortie/example && logger 'ortie stored a token'"
  # v3: a chain reading the token, kept in a variable since the store consumes stdin
  storage.write.command = "json=$(cat); printf '%s' \"$json\" | pass insert -m -f ortie/example && notify-send Ortie \"Token stored, expires in $(printf '%s' \"$json\" | jq .expires_in)s\""
  ```

  The write command failing fails the command, so keep a chained step from failing (`|| true`). An error hook (`hooks.on-issue.error`, `hooks.on-refresh.error`) becomes a check of the exit status of the command, and of its `code` under `--json` (see Output):

  ```sh
  ortie token refresh || notify-send Ortie "Refresh failed"
  ```

- **`auto-refresh` defaults to true**: `token show` refreshes an expired token, or re-acquires it on the client credentials grants, where v2 printed it expired. An account wanting the v2 behaviour sets it:

  ```toml
  [accounts.example]
  auto-refresh = false
  ```

- **`-c` takes one path per flag.** A value is no longer split on `:`, so a Windows path such as `C:\Users\me\config.toml` stays whole. Repeat the flag to merge several files, the first the base:

  ```sh
  # v2
  ortie -c public.toml:private.toml token show
  # v3
  ortie -c public.toml -c private.toml token show
  ```

- **`ORTIE_CONFIG` is no longer read**: left in the environment, it is ignored and the default paths are read. Pass `-c "$ORTIE_CONFIG"` for one path, and one `-c` per path for what was a `:`-separated list, in a script or a systemd unit.
- **Every default path is read, as layers.** $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml and $HOME/.ortierc are each deep-merged on top of the ones before, where v2 read the first one found. An rc file left beside a config.toml now overrides it: delete the one you do not use.
- **Several `default = true` accounts are refused**, naming them ("Several accounts are marked default (a, b), keep `default = true` on one, or name one with `-a <NAME>`"), where v2 picked one at random. Keep it on one account.
- **`-a default` names the account called `default`**, where v2 read it as the account marked `default = true`. Omit `-a` to take the marked one, in scripts, aliases and systemd units; an account actually named `default` is now reachable. Without one, the run fails coded `account-missing` (see Output), with the hint "omit `-a` to take the account marked default".

  ```sh
  # v2
  ortie -a default token show
  # v3
  ortie token show
  ```

- **`tls` is a table**, as in every Pimalaya CLI: `tls.provider` names the implementation (`rustls` or `native-tls`) and `tls.rustls.crypto` the crypto provider of rustls (`ring` or `aws`), each the first the build carries when omitted, and the new `tls.cert` a PEM certificate to trust. The string v2 read is refused by name, the error giving the keys its value becomes:

  ```toml
  # v2
  tls = "rustls-aws"
  # v3
  tls.provider = "rustls"
  tls.rustls.crypto = "aws"
  ```

  `tls = "native-tls"` becomes `tls.provider = "native-tls"`, `tls = "rustls-ring"` `tls.provider = "rustls"` with `tls.rustls.crypto = "ring"`, and `tls = "auto"` is deleted.

- **A URL takes an ASCII host only**: write an internationalised host in Punycode, in the endpoints, the redirection and the resources.

  ```toml
  # v2
  endpoints.token = "https://bücher.example/token"
  # v3
  endpoints.token = "https://xn--bcher-kva.example/token"
  ```

### Commands

- **`auth get` opens no browser**: it prints the authorization link, or the verification URI and the user code, for you to open; at a terminal it still waits for the redirection and stores the token. A script that relied on the browser opening runs `xdg-open` (or `open`, `start`) on the `authorizationUri` of `auth get --json`, then `auth listen` or `auth resume` as before.
- **`auth resume --pkce` refuses a verifier** that is not 43 to 128 unreserved characters (RFC 7636 section 4.1), and `--state` an empty state. Pass back the values `auth get` printed, which always pass.
- **`ortie repl` refuses a line setting `-a`, `-c` or a log flag**, naming it: the session fixes them when it starts, so pass them to `ortie repl` itself. A line is split as a shell does (quotes, escapes, `#` comments), takes subcommand prefixes, and takes `--json` for that line only. The prompt is `ortie> `. A line takes any command of the binary, `account check` and `completion bash` included, a `repl` or `mcp` line being refused coded `usage-invalid` ("Cannot start a session inside a session").

### Output

- **`--json` keys are camelCase**; the configuration keeps its kebab-case keys, and the stored token keeps the OAuth wire names (`access_token`, `expires_in`). Rename the keys a script reads:

  | Command | v2 | v3 |
  |---------|----|----|
  | `token show` | `access_token` | `accessToken` |
  | `token inspect` | `token_type`, `issued_at`, `with_refresh_token` | `tokenType`, `issuedAt`, `withRefreshToken`, and `expiresAt` for `expires_in` (below) |
  | `auth get` (authorization code) | `authorization_uri`, `pkce_code_verifier` | `authorizationUri`, `pkceCodeVerifier`, and the new `redirectUri` |
  | `auth get` (device) | `device_code`, `user_code`, `verification_uri`, `verification_uri_complete`, `expires_in` | `deviceCode`, `userCode`, `verificationUri`, `verificationUriComplete`, `expiresIn` |

- **`token inspect --json` prints `issuedAt` as an RFC 3339 instant** in UTC (`2026-11-15T09:00:00Z`), where v2 printed `issued_at` as epoch seconds.
- **`token inspect --json` prints when the token expires, `expiresAt`**, where v2 printed `expires_in`, the lifetime granted at issue, unchanged however long ago that was: v2 printed `"expires_in": 3600` for a token issued at 09:00 and read at 09:50, v3 prints `"expiresAt": "2026-11-15T10:00:00Z"`, an instant in the past once the token expired. `expiresAt` is `null` when the store holds no issue time or no lifetime, and the text then reads `Expires: unknown` where it read `in 1h`. A script wanting the seconds left subtracts the current time, one wanting the v2 lifetime subtracts `issuedAt` (jq lines below).
- **A stored token prints a receipt.** `token refresh`, `auth resume`, and an `auth get` that issues the token (a client credentials grant, or an interactive one it completed) print `{"expiresIn": 3599, "withRefreshToken": true}` under `--json`, where v2 printed `{"message": "Access token successfully refreshed (expires in 1h)"}`. A script checking the sentence checks the exit status instead, 0 on success. The text form keeps its sentence.
- **A failure carries a stable `code`** in the failure object under `--json`, beside `error`, `sources` and `backtrace`: `permission-denied`, `rate-limited`, `grant-invalid`, `client-invalid`, `client-unauthorized`, `scope-invalid`, `device-code-expired`, `state-mismatch`, `token-missing`, `endpoint-missing`, `account-missing`, `account-default-missing`, `account-default-several` or `usage-invalid`, `null` for any other failure. Match on `code` rather than on the wording of `error`. An empty token storage now fails coded `token-missing`, where v2 failed with a JSON parse error.
- **An OAuth refusal names its code as the server spelled it**, `Refresh access token error (code invalid_grant)` where v2 printed `(code InvalidGrant)`, and `(unregistered code)` for a code Ortie does not know. A script grepping the text moves to `code` under `--json` (`invalid_grant` is `grant-invalid`).
- **A refused command line is a failure object under `--json`**: an unknown flag or command, a missing argument or a bad value prints the failure object coded `usage-invalid` on stdout and exits 2, where v2 printed clap's text on stderr. Without `--json` clap's text stays on stderr, and the exit status is 2 either way.
- **`ortie repl` prints its errors on stdout**, interactive or piped, as `ortie-error` objects under `--json`, where v2 wrote them on stderr: a piped session reads one JSON value per line on stdout, payload or failure, each failure carrying its `code` (`usage-invalid` for a line that does not parse). At a terminal, a line that does not parse keeps clap's text on stderr, and a `--json` session prints help on stderr.
- **Piped `--json` output is one value per line**, and every text output ends with a newline, `token show` included: `$(ortie token show)` strips it, as do the secret commands of the Pimalaya tools.
- **The text of `token inspect` and `auth get` changed** to aligned `Label: value` fields, one scope per line. A script parsing it moves to `--json`.
- **`json-schema` stamps an `$id`** naming the contract version on each schema and requires the keys a command always prints. `ortie-configure` is gone; `ortie-token-refresh`, `ortie-auth-resume`, `ortie-token-revoke`, `ortie-auth-listen` and `ortie-error` are new.

Reading the new output with jq:

```sh
# the access token
ortie token show --json | jq -r .accessToken
# when the token was issued, as the epoch seconds v2 printed
ortie token inspect --json | jq '.issuedAt | if . then fromdateiso8601 else null end'
# the seconds left before the token expires, negative once it expired
ortie token inspect --json | jq '.expiresAt | if . then fromdateiso8601 - now | floor else null end'
# the lifetime granted at issue, the expires_in v2 printed
ortie token inspect --json | jq 'if .expiresAt and .issuedAt then (.expiresAt | fromdateiso8601) - (.issuedAt | fromdateiso8601) else null end'
# the seconds the refreshed token lives, or why the refresh failed
ortie token refresh --json | jq -r 'if .error then "failed: \(.code)" else .expiresIn end'
# start a new flow when the refresh token was refused
[ "$(ortie token refresh --json | jq -r .code)" = grant-invalid ] && ortie auth get
```

### Building

- **`vendored` is a default cargo feature**, as in the other Pimalaya CLIs: a build with `--features native-tls` links a vendored OpenSSL. A packager linking the system one builds with `--no-default-features --features native-tls`.

### New commands

- **`ortie token revoke`** revokes the stored refresh token, or the access token when none is stored (RFC 7009), at the new `endpoints.revocation`, and under `--json` prints which one went, `{"revoked": "refresh-token"}` or `{"revoked": "access-token"}`.
- **`ortie auth listen`** catches the redirection of a flow a non-interactive or `--json` `auth get` handed off, then completes it as `auth resume` does.
- **`ortie mcp`** serves the grants of one account to an agent over the Model Context Protocol; no tool returns a token. Started without a flag it lists the tools that read alone, `token_inspect` and `auth_status`: an MCP client configuration driving the grants passes `--allow-write` for `auth_get`, `auth_resume` and `token_refresh`, and `--allow-revoke` on top of it for `token_revoke`.
- **`ortie account list`** lists the accounts of the configuration, and **`ortie account check`** proves the account's storage reads its token and its endpoints answer HTTP, one row each, exiting 3 when one fails.

## From v1 to v2

v2 rewrote Ortie as a pure CLI over io-oauth, with a wizard, the device and client credentials grants and PKCE by default. A v1 configuration loads as it is, unless it uses the keyring storage or a notification hook, but the token v1 stored no longer parses: run `auth get` once after upgrading.

This section was built from the CHANGELOG and the tagged sources of v1.1.0 and v2.0.0, and the last v2 release (2.3.0) where a 2.x release changed a v1 behaviour again; the stored-token item was reproduced with the current (v3) binary, which reads `issued_at` the same way, the others were read from the sources and not run.

- **Run `ortie auth get` once after upgrading.** v1 stored the token with an `issued_at` object (`{"secs_since_epoch": …, "nanos_since_epoch": …}`), which v2 reads as epoch seconds, so `token show` fails with "Parse access token from command error" and "invalid type: map, expected u64". A new `auth get` stores a token v2 reads. To keep the stored refresh token instead, drop the field through your store, then run `ortie token refresh`, since a token without `issued_at` counts as unexpired and `token show` would not refresh it:

  ```sh
  pass show ortie/example | jq -c 'del(.issued_at)' | pass insert -m -f ortie/example
  ortie token refresh
  ```

- **The keyring storage is gone** (since 1.1.0): `storage.read.keyring`, `storage.write.keyring` and `client-secret.keyring` are refused. Use a command reading and writing your keyring, such as `secret-tool`, `security` or `pass`:

  ```toml
  # v1.0
  client-secret.keyring = "example"
  storage.read.keyring = "ortie-example"
  storage.write.keyring = "ortie-example"

  # v2
  client-secret.command = "secret-tool lookup service ortie-example-secret"
  storage.read.command = "secret-tool lookup service ortie-example"
  storage.write.command = "secret-tool store --label ortie-example service ortie-example"
  ```

- **PKCE is on by default** (S256), where v1 defaulted `pkce` to false. A server refusing the PKCE parameters needs `pkce = false`; `pkce = true` and `false` keep their meaning, and `"s256"` or `"plain"` name the method.
- **A string command runs through the shell** (`/bin/sh -c` on Unix, `cmd /C` on Windows), where v1 split it on whitespace and ran it directly. Quotes, `$VAR`, `~`, pipes and `;` in a string are now the shell's; an exec-style array still runs directly. A hook relying on `$EXPIRES_IN` and the other hook variables being expanded inside an array switches to a string:

  ```toml
  # v1
  hooks.on-refresh.success.command = ["logger", "ortie refreshed, expires in $EXPIRES_IN"]
  # v2
  hooks.on-refresh.success.command = "logger \"ortie refreshed, expires in $EXPIRES_IN\""
  ```

- **A hook's `notify` table needs the `notify` cargo feature**, which v1 built by default and v2 does not: a default v2 build, which the v2 README says the pre-built binaries are, refuses it as an unknown field. Rebuild with `--features notify`, or call `notify-send` from the hook's `command`. v3 removes the hooks altogether (see From v2 to v3).
- **`--debug` and `--trace` are `--log-level debug` and `--log-level trace`**, beside a new `--log-file <PATH>`; `--quiet` (alias `--silent`) is gone, logs being off unless `--log-level` or `RUST_LOG` asks for them.

  ```sh
  # v1
  ortie token show --debug 2>/tmp/ortie.log
  # v2
  ortie token show --log-level debug --log-file /tmp/ortie.log
  ```

- **`-c` splits its value on `:`** in v2, so a Windows path such as `C:\Users\me\config.toml` breaks; repeat `-c` instead. v3 stops splitting.
- **`manuals` and `completions` are `manual` and `completion`** (since 2.3.0, the plural kept as a hidden alias), and print to stdout: `ortie manuals ./man` is `ortie manual --dir ./man`, and `ortie completions bash`, which v1 wrote to ./, is `ortie completion bash > ortie.bash`.
- **`token inspect --json` prints `issued_at` as epoch seconds** (`null` when unknown) where v1 printed the `issued_at` object, and since 2.3.0 no longer prints `access_token` nor `refresh_token`, `with_refresh_token` telling whether one is stored. Read the token from `token show`. v3 renames these keys again (see From v2 to v3).
- **The library target is gone**: a Rust program depending on the `ortie` crate moves to [io-oauth](https://github.com/pimalaya/io-oauth).
- **The cargo features `command` and `keyring` are gone** (storage commands always build), and `notify` leaves the default set; the default TLS provider stays `rustls-ring`. A packager passing `--features command` drops it.

## From v0.1 to v1

v1 grouped the hooks under one `hooks` table. This section was built from the CHANGELOG and the tagged sources of v0.1.0 and v1.0.0, read and not run.

- **`on-issue-access-token` is `hooks.on-issue`, and `on-refresh-access-token` is `hooks.on-refresh`.** v1 refuses the old keys as unknown fields; the `success` and `error` tables under them, and their `command` and `notify`, keep their shape.

  ```toml
  # v0.1
  on-issue-access-token.success.command = ["notify-send", "Ortie", "Token issued"]
  on-refresh-access-token.error.notify.summary = "Ortie"
  on-refresh-access-token.error.notify.body = "[$ERROR] Refresh access token error"

  # v1
  hooks.on-issue.success.command = ["notify-send", "Ortie", "Token issued"]
  hooks.on-refresh.error.notify.summary = "Ortie"
  hooks.on-refresh.error.notify.body = "[$ERROR] Refresh access token error"
  ```

- **An account without `endpoints.redirection` listens on a random port** of 127.0.0.1 and sends it as the redirect URI, where v0.1 sent none and listened on port 80. A provider registration expecting the bare loopback keeps working when it follows RFC 8252 (any port); otherwise set `endpoints.redirection` to the registered URI.
- **The default cargo features changed** from `command` and `rustls-ring` to `command`, `native-tls`, `keyring` and `notify`, and back to `rustls-ring` in 1.1.0 (see From v1 to v2). A build from source wanting a given TLS provider names it.
