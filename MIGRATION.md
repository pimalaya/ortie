# Migration guide

## From v2 to v3

v3 aligns Ortie on the other Pimalaya CLIs. A v2 configuration loads as it is unless it holds a `hooks` table, sets `tls` as a string, marks several accounts default or writes a host in Unicode. The [CHANGELOG](./CHANGELOG.md) lists every change; these are the ones to act on.

### Configuration

- **The wizard is gone**, with `ortie configure` and dynamic client registration. Start from a provider account of the [README](./README.md#configuration) or from [config.sample.toml](./config.sample.toml). A Fastmail account the wizard registered keeps working.
- **The `hooks` table is refused**, and the `notify` cargo feature is gone. `storage.write.command` runs on every token stored, and receives it as JSON on stdin (`access_token`, `expires_in`, `refresh_token`, `scope`): chain what a success hook did after it, and check the exit status of the command for what an error hook did.

  ```toml
  # v2
  hooks.on-refresh.success.command = "logger 'ortie refreshed a token'"
  # v3
  storage.write.command = "pass insert -m -f ortie/example && logger 'ortie stored a token'"
  ```

- **`auto-refresh` defaults to true**: `token show` refreshes an expired token. Set `auto-refresh = false` to print it expired as v2 did.
- **`-c` takes one path per flag**, no longer split on `:`: write `-c public.toml -c private.toml`.
- **`ORTIE_CONFIG` is no longer read**: pass `-c "$ORTIE_CONFIG"`.
- **Every default path is read**, each deep-merged on top of the previous one, where v2 read the first found: delete the file you do not use.
- **Several `default = true` accounts are refused**: keep it on one.
- **`-a default` names the account called `default`**: omit `-a` to take the account marked default.
- **`tls` is a table**:

  ```toml
  # v2
  tls = "rustls-aws"
  # v3
  tls.provider = "rustls"
  tls.rustls.crypto = "aws"
  ```

  `"native-tls"` becomes `tls.provider = "native-tls"`, `"rustls-ring"` `tls.provider = "rustls"` with `tls.rustls.crypto = "ring"`; delete `tls = "auto"`.

- **A URL takes an ASCII host only**: write an internationalised host in Punycode (`https://xn--bcher-kva.example` for `bücher.example`).

### Commands

- **`auth get` opens no browser**: open the link it prints. A script opens the `authorizationUri` of `auth get --json` itself.
- **`auth resume --pkce` refuses a malformed verifier** and `--state` an empty state: pass back what `auth get` printed.
- **`ortie repl` splits its lines as a shell does** and refuses a line setting `-a`, `-c` or a log flag: pass those to `ortie repl` itself.

### Output

- **`--json` keys are camelCase**: `accessToken`, `tokenType`, `issuedAt`, `withRefreshToken`, `authorizationUri`, `pkceCodeVerifier`, `deviceCode`, `userCode`, `verificationUri`, `verificationUriComplete`, `expiresIn`. The configuration and the stored token keep their own names.
- **`token inspect --json` prints instants**: `issuedAt` in RFC 3339 instead of epoch seconds, and `expiresAt` instead of `expires_in`.

  ```sh
  # the seconds left, negative once expired
  ortie token inspect --json | jq '.expiresAt | if . then fromdateiso8601 - now | floor else null end'
  ```

- **Storing a token prints a receipt**, `{"expiresIn", "withRefreshToken"}`, instead of `{"message"}`: check the exit status rather than the sentence.
- **A failure carries a stable `code`** under `--json`: match on it rather than on the wording of `error`. An OAuth refusal names its code as the server spelled it, `(code invalid_grant)`.
- **A refused command line exits 2 coded `usage-invalid`**, clap's refusals and the ones Ortie makes after parsing alike, printed on stdout under `--json`.
- **`ortie repl` prints its errors on stdout**, as `ortie-error` objects under `--json`.
- **The text output changed**: aligned `Label: value` fields, durations such as `2d 4h`, a newline after `token show`. A script parsing text moves to `--json`.

### Building

- **`vendored` is a default feature**: build with `--no-default-features --features native-tls` to link the system OpenSSL.

## From v1 to v2

v2 rewrote Ortie as a pure CLI over io-oauth.

- **Run `ortie auth get` once after upgrading**: the token v1 stored no longer parses.
- **The keyring storage is gone**: read and write the keyring through commands.

  ```toml
  # v1
  storage.read.keyring = "ortie-example"
  # v2
  storage.read.command = "secret-tool lookup service ortie-example"
  storage.write.command = "secret-tool store --label ortie-example service ortie-example"
  ```

- **PKCE is on by default** (S256): set `pkce = false` for a server refusing it.
- **A string command runs through the shell**; an array still runs directly.
- **A hook's `notify` table needs the `notify` cargo feature**, off by default.
- **`--debug` and `--trace` are `--log-level debug` and `--log-level trace`**, and `--quiet` is gone, logs being off by default.
- **`manuals` and `completions` are `manual` and `completion`**, printing to stdout: `ortie manual --dir ./man`, `ortie completion bash > ortie.bash`.
- **The library target is gone**: depend on [io-oauth](https://github.com/pimalaya/io-oauth).
- **The cargo features `command` and `keyring` are gone**, and `notify` leaves the default set.
