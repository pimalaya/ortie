---
cairn: spec
capability: output
status: current
---

# Output

Everything the user asked for goes to stdout, data and errors alike, distinguished only by the exit code; stderr carries the logs and the welcome a missing configuration meets a person with. `--json` switches the data commands to machine-readable objects, and this capability is what describes those objects.

### Requirement: A data command prints a named output type
Each command printing a value SHALL print one type named after the invocation and suffixed `Output`, deriving `Display`, `Serialize` and `JsonSchema`, so the human rendering, the machine rendering and the schema are three faces of one declaration. A sentence SHALL be the `Display` of such a type, never a value of its own.

### Requirement: A schema per structured-output command
Ortie SHALL provide a `json-schema` command printing the JSON Schema of a command's `--json` payload, and writing one file per command into `--dir` when given one, creating the directory when it does not exist and overwriting the schemas already in it. Each file SHALL be named after the CLI invocation it describes, the command path joined with hyphens and prefixed `ortie-`, mirroring how the man pages are named.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, as printed (the serialize contract), so the description cannot drift from the emission: a key the type always prints is required, `null` or not. Where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Only Ortie's own types are described
Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth type, so the machine contract stays a decision of this repository and needs no release of the library below it. The family's shared shape from pimalaya-cli is the exception: the failure object `ErrorReport`. The account outputs (`AccountListOutput`, `AccountShowOutput`, `AccountCheckOutput`, `AccountCheckEvent`) are Ortie's own, Ortie having no backend for the family's shapes to describe.

### Requirement: Commands carrying a schema
Every command printing a value SHALL carry one key: `ortie-account-list`, `ortie-account-show`, `ortie-account-check`, `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-listen` and `ortie-auth-resume`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help, and neither does the REPL: it dispatches the commands of the binary and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload. A command printing events before its payload SHALL register their type in the registry's `events` map under its own key, written as `<key>.events`: `ortie-account-check` for the `check-finished` events of `account check` (`AccountCheckEvent`, Ortie's own), the registry check covering both maps.

### Requirement: One command, one schema
A command emitting more than one payload shape SHALL print them through a single type describing the alternatives, told apart by their keys, so its key maps to one schema. `auth get` emits the authorization-code handoff, the device-authorization handoff, or the receipt of a token its grant issued in-process, and all three are variants of that one type.

### Requirement: A stored token answers a receipt
A command storing a new access token (`token refresh`, `auth listen`, `auth resume`, and `auth get` when its grant issues the token in-process) SHALL print `TokenIssuedOutput`, `{account, expiresIn, withRefreshToken}`: the name of the account the token was stored for, first, as a write names its target; the token's lifetime in seconds as the server stated it, `null` when it stated none; and whether a refresh token came with it. The token itself SHALL NOT appear. Its text form SHALL read "Access token of account <name> successfully issued" or "refreshed", with the expiry.

#### Scenario: A script reads when a refreshed token expires
- GIVEN an account `example` holding a refresh token
- WHEN `ortie token refresh --json` runs
- THEN it prints `{"account": "example", "expiresIn": 3599, "withRefreshToken": true}` for a token the server issued for 3599 seconds with a new refresh token

### Requirement: A key names a command that exists
A test SHALL run pimalaya-cli's registry check over the command tree: every runnable command described, bare `ortie` and `ortie repl` aside, and every key but `ortie-error` naming a command, so renaming or removing a subcommand cannot leave a schema behind describing an invocation nobody can type, nor a new one print an undescribed value.

### Requirement: Nothing bypasses the printer under JSON
Every write to stdout that is not the printer's SHALL be guarded by the JSON mode, so a `--json` run emits the payload and nothing else. The handoff instructions and the wait notices of `auth get` are human text on the human path only.

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `ortie-error`, its `code` typed by the code vocabulary and pimalaya-cli's `usage-invalid`. Every generated schema SHALL carry an `$id` naming the contract version, the major version of Ortie. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`. Configuration types SHALL stay kebab-case: what the loader reads is not what the printer emits.

### Requirement: Shared shapes and spellings
Values Ortie defines (`revoked`, the MCP `kind` and `status`, the codes) SHALL be kebab-case; values a protocol defines (the token type, the scope, the OAuth codes) SHALL keep its spelling. `token inspect` SHALL print `issuedAt` and `expiresAt` as RFC 3339 instants in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

#### Scenario: The kind of a revoked token
- GIVEN a store holding a refresh token and a revocation endpoint accepting it
- WHEN `ortie token revoke --json` runs
- THEN it prints `{"revoked": "refresh-token"}`

### Requirement: Text output ends with a newline
Every command's text rendering SHALL end with a newline, `token show` included, so the shell prompt starts on a line of its own. The REPL SHALL add none of its own: interactive, its prompt `ortie> ` follows the output on a line of its own; piped, results follow each other line after line.

### Requirement: A text record prints as aligned fields
A text rendering holding a record, `token inspect` and the `auth get` authorization and device requests, SHALL print it through pimalaya-cli's `fields::Fields`: one `Label: value` line per field, values aligned two spaces past the longest label's colon, a list such as the scopes one item per line under the first. A link or a command for the user to copy SHALL follow its sentence after a blank line, indented two spaces, as the welcome and clap's usage lines are. A value a server wrote, the token type, a scope, the user code and the verification URIs, SHALL be defused by pimalaya-cli's `sanitize::text` first.

### Requirement: A duration prints in jiff's friendly form
A duration the text output prints, the age and the remaining lifetime of `token inspect`, the lifetime of the issued-token receipt and of the device request, SHALL be printed in whole seconds through jiff's friendly format with its default printer: compact designators (`d`, `h`, `m`, `s`), a space between units, units balanced up to days of 24 hours and never to weeks, months nor years, `0s` when empty. A duration too long for a jiff span SHALL stop at hours. The JSON prints seconds or instants, never this text.

#### Scenario: A token issued two days ago
- GIVEN a store holding a token issued 2 days, 4 hours and 30 minutes ago
- WHEN `ortie token inspect` runs
- THEN its `Issued` field reads `2d 4h 30m ago`, where v2 printed `2days 4h 30m ago`

### Requirement: A refused command line exits 2
The command line SHALL be parsed through pimalaya-cli's `clap::parse`: help and version print as clap prints them, and a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value) SHALL exit 2 before any configuration is read. With a `--json` word before any `--`, it SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without its `error: ` prefix nor styles, `sources` empty and `backtrace` null, and nothing on stderr; otherwise clap's rendering on stderr and nothing on stdout. Exit 1 stays a failure object.

#### Scenario: An unknown flag under JSON
- WHEN `ortie --json token show --nope` runs
- THEN it prints one failure object coded `usage-invalid`, its `error` naming `--nope`, on stdout and exits 2

### Requirement: A log file that cannot be opened fails the command
A `--log-file` Ortie cannot open or create SHALL fail the command before it runs, as any other failure: the failure object on stdout, naming the path, and exit 1, never a panic.

### Requirement: A failure carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, found anywhere in the error chain or read from io-oauth's typed client error or pimalaya-config's account resolution error. A failure with no code SHALL print `"code": null`; the text output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. An OAuth refusal SHALL name its code in its message as the server spelled it (`invalid_grant`), a code io-oauth does not register reading `unregistered code`.

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar, raised by pimalaya-cli; or Ortie refuses the line after parsing it (a command or a flag the account's grant has no use for, `--handoff` among them, an `auth listen` whose `--redirect-uri` or handoff `redirectUri` is no loopback redirection to bind, an empty device code, a redirected URI that does not parse), exiting 2 as well, and the MCP `auth_resume` refusing such a URI as a tool error; nothing ran |
| `permission-denied` | the grant was refused to the user or the client: `access_denied` at the authorization redirect or in the device flow, `authorization_declined` (Microsoft Entra), a bare HTTP 403 |
| `rate-limited` | a bare HTTP 429; Ortie retries nothing |
| `grant-invalid` | `invalid_grant`: the authorization code, the refresh token or the device code was refused, has expired or was revoked; run `auth get` again |
| `client-invalid` | `invalid_client`: the client secret, certificate or assertion was refused |
| `client-unauthorized` | `unauthorized_client`: the client may not use the grant |
| `scope-invalid` | `invalid_scope` |
| `device-code-expired` | the device code expired before the user approved it, by the server's `expired_token` or by the local deadline |
| `state-mismatch` | the authorization redirect carries no `state`, or another than the request sent |
| `token-missing` | the account stores no token (its read command prints nothing, exiting 0, or non-zero without a word), or no refresh token where a refresh needs one; run `auth get` |
| `token-expired` | the `token` row of `account check`: the stored token expired and holds no refresh token, on a grant other than the client credentials ones, which run again by themselves; run `auth get` |
| `endpoint-missing` | the account configures no endpoint the grant needs (authorization, token, device authorization, revocation), or no redirection an `auth listen` without `--redirect-uri` binds; exit 1 |
| `redirection-unsupported` | the `endpoints.redirection` an `auth listen` without `--redirect-uri` would bind is not an http URI on a loopback host; exit 1, `auth resume` with the redirected URI finishing the flow |
| `account-missing` | no account has the name `-a` gave, `default` included |
| `account-default-missing` | no account is marked `default = true`, and `-a` named none |
| `account-default-several` | several accounts are marked `default = true`, and `-a` named none |
| `stdin-taken` | a command asked for its input on stdin (`--handoff -`) inside a REPL or MCP session, which reads its own lines there |

Any other OAuth error code (`invalid_request`, `unsupported_grant_type`, `unsupported_token_type`, `invalid_target`, `server_error`, `temporarily_unavailable`, an unknown one) SHALL carry no code.

#### Scenario: A revoked refresh token
- GIVEN an account whose stored refresh token the server answers `invalid_grant`
- WHEN `ortie --json token refresh` runs
- THEN it exits 1 printing a failure object coded `grant-invalid`, its `error` naming `invalid_grant`
