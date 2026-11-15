# Migration guide

## From v2 to v3

v3 brings Ortie onto the standard the Pimalaya CLIs share: no configuration wizard, one configuration path per `-c`, camelCase keys and receipts under `--json`, coded failures, and hooks folded into the storage commands. A v2 configuration loads as it is, unless it holds a `hooks` table, marks several accounts default, or writes a host in Unicode.

The [CHANGELOG](./CHANGELOG.md) lists every change; these are the ones to act on.

### Configuration and first run

- **The wizard is gone.** `ortie configure` (alias `wizard`), its `ortie-configure` schema and dynamic client registration (RFC 7591) no longer exist, and bare `ortie` prints the help. Write an account from [config.sample.toml](./config.sample.toml), which releases and packages ship as share/doc/ortie/config.sample.toml: uncomment the section of your provider (Google, Microsoft Graph, Microsoft IMAP and SMTP, Fastmail, or a custom application) and fill its storage commands. Run at a terminal without a configuration, Ortie prints the commands copying it in place. A Fastmail account the wizard registered keeps working; a new one takes the registered client of the sample.
- **The `hooks` table is refused**, by name: "The `hooks` table was removed in 3.0.0: chain a command after `storage.write.command`, which receives the token on every issue and refresh". `storage.write.command` runs on every token `auth get`, `auth resume`, `auth listen`, `token refresh` and an auto-refreshing `token show` store, so a command chained after it does what a success hook did. The `notify` tables and the `notify` cargo feature went with the hooks: call `notify-send` or any notifier from the command. The variables v2 exported (`ACCESS_TOKEN`, `EXPIRES_IN`, `REFRESH_TOKEN` and so on) are gone; the token JSON on the command's stdin carries the same values, under the OAuth wire names (`access_token`, `expires_in`, `refresh_token`, `scope`).

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
- **`ortie repl` refuses a line setting `-a`, `-c` or a log flag**, naming it: the session fixes them when it starts, so pass them to `ortie repl` itself. A line is split as a shell does (quotes, escapes, `#` comments), takes subcommand prefixes, and takes `--json` for that line only. The prompt is `ortie> `.

### Output

- **`--json` keys are camelCase**; the configuration keeps its kebab-case keys, and the stored token keeps the OAuth wire names (`access_token`, `expires_in`). Rename the keys a script reads:

  | Command | v2 | v3 |
  |---------|----|----|
  | `token show` | `access_token` | `accessToken` |
  | `token inspect` | `token_type`, `issued_at`, `expires_in`, `with_refresh_token` | `tokenType`, `issuedAt`, `expiresIn`, `withRefreshToken` |
  | `auth get` (authorization code) | `authorization_uri`, `pkce_code_verifier` | `authorizationUri`, `pkceCodeVerifier`, and the new `redirectUri` |
  | `auth get` (device) | `device_code`, `user_code`, `verification_uri`, `verification_uri_complete`, `expires_in` | `deviceCode`, `userCode`, `verificationUri`, `verificationUriComplete`, `expiresIn` |

- **`token inspect --json` prints `issuedAt` as an RFC 3339 instant** in UTC (`2026-11-15T09:00:00Z`), where v2 printed `issued_at` as epoch seconds.
- **A stored token prints a receipt.** `token refresh`, `auth resume`, and an `auth get` that issues the token (a client credentials grant, or an interactive one it completed) print `{"expiresIn": 3599, "withRefreshToken": true}` under `--json`, where v2 printed `{"message": "Access token successfully refreshed (expires in 1h)"}`. A script checking the sentence checks the exit status instead, 0 on success. The text form keeps its sentence.
- **A failure carries a stable `code`** in the failure object under `--json`, beside `error`, `sources` and `backtrace`: `permission-denied`, `rate-limited`, `grant-invalid`, `client-invalid`, `client-unauthorized`, `scope-invalid`, `device-code-expired`, `state-mismatch`, `token-missing`, `endpoint-missing` or `usage-invalid`, `null` for any other failure. Match on `code` rather than on the wording of `error`. An empty token storage now fails coded `token-missing`, where v2 failed with a JSON parse error.
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
# the seconds the refreshed token lives, or why the refresh failed
ortie token refresh --json | jq -r 'if .error then "failed: \(.code)" else .expiresIn end'
# start a new flow when the refresh token was refused
[ "$(ortie token refresh --json | jq -r .code)" = grant-invalid ] && ortie auth get
```

### New commands

- **`ortie token revoke`** revokes the stored refresh token, or the access token when none is stored (RFC 7009), at the new `endpoints.revocation`.
- **`ortie auth listen`** catches the redirection of a flow a non-interactive or `--json` `auth get` handed off, then completes it as `auth resume` does.
- **`ortie mcp`** serves the grants of one account to an agent over the Model Context Protocol; no tool returns a token, and `token_revoke` is listed only under `--allow-revoke`.
