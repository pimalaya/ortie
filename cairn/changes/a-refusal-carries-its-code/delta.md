---
cairn: delta
change: a-refusal-carries-its-code
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A refused command line exits 2
(output) The command line SHALL be parsed through pimalaya-cli's `clap::parse`: help and version print as clap prints them, and a command line the grammar refuses (an unknown flag or command, a missing argument, a bad value) SHALL exit 2 before any configuration is read. With a `--json` word before any `--`, it SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without its `error: ` prefix nor styles, `sources` empty and `backtrace` null, and nothing on stderr; otherwise clap's rendering on stderr and nothing on stdout. Exit 1 stays a failure object.

#### Scenario: An unknown flag under JSON
- WHEN `ortie --json token show --nope` runs
- THEN it prints one failure object coded `usage-invalid`, its `error` naming `--nope`, on stdout and exits 2

### Requirement: A failure carries a stable code
(output) A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, found anywhere in the error chain or read from io-oauth's typed client error. A failure with no code SHALL print `"code": null`; the text output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. An OAuth refusal SHALL name its code in its message as the server spelled it (`invalid_grant`).

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar, raised by pimalaya-cli; nothing ran |
| `permission-denied` | the grant was refused to the user or the client: `access_denied` at the authorization redirect or in the device flow, `authorization_declined` (Microsoft Entra), a bare HTTP 403 |
| `rate-limited` | a bare HTTP 429; ortie retries nothing |
| `grant-invalid` | `invalid_grant`: the authorization code, the refresh token or the device code was refused, has expired or was revoked; run `auth get` again |
| `client-invalid` | `invalid_client`: the client secret, certificate or assertion was refused |
| `client-unauthorized` | `unauthorized_client`: the client may not use the grant |
| `scope-invalid` | `invalid_scope` |
| `device-code-expired` | the device code expired before the user approved it, by the server's `expired_token` or by the local deadline |
| `state-mismatch` | the authorization redirect carries no `state`, or another than the request sent |
| `token-missing` | the account stores no token, or no refresh token where a refresh needs one; run `auth get` |
| `endpoint-missing` | the account configures no endpoint the grant needs (token, device authorization, revocation) |

Any other OAuth error code (`invalid_request`, `unsupported_grant_type`, `unsupported_token_type`, `invalid_target`, `server_error`, `temporarily_unavailable`, an unknown one) SHALL carry no code.

#### Scenario: A revoked refresh token
- GIVEN an account whose stored refresh token the server answers `invalid_grant`
- WHEN `ortie --json token refresh` runs
- THEN it exits 1 printing a failure object coded `grant-invalid`, its `error` naming `invalid_grant`

## MODIFIED Requirements

### Requirement: The JSON contract is described and versioned
(output) `json-schema` SHALL describe the failure object under the key `ortie-error`, its `code` typed by the code vocabulary and pimalaya-cli's `usage-invalid`. Every generated schema SHALL carry an `$id` naming the contract version, the major version of Ortie. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: A bare error status is named
(auth) When the token, device authorization or revocation endpoint answers an error status without an OAuth error body, the command SHALL fail naming the HTTP status, coded `permission-denied` for a 403 and `rate-limited` for a 429.

### Requirement: Interactive versus piped output
(repl) Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, its text form hinting `Restart the session with --log-level to enable more verbose logs` while debug logs are off, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as the failure object coded `usage-invalid`, its message without clap's `error: ` prefix. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line. The help's usage line SHALL name `ortie` before the command, as on the command line.

#### Scenario: A command's help in a session
- GIVEN a running `ortie repl` session
- WHEN `help token show` is entered
- THEN the help's usage line reads `Usage: ortie token show [OPTIONS]`

### Requirement: A tool's output is the command's payload
(mcp) A successful call SHALL return its JSON value as `structuredContent` and as text, described as `outputSchema` by the json-schema registry entry of the command it runs, or by the schema of the `auth_get` and `auth_status` answers, each derived as printed (the serialize contract): a key the answer always prints is required, `null` or not. A failed call SHALL return `isError: true` with the `ortie-error` object, its `code` from the vocabulary. An argument the tool does not declare SHALL be refused naming the declared ones, and a missing or non-string one naming it, each coded `usage-invalid`; an unknown or unlisted tool SHALL be a JSON-RPC error (-32602).

## REMOVED Requirements

None.
