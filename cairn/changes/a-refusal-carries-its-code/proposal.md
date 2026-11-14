---
cairn: change
id: a-refusal-carries-its-code
status: landed
created: 2026-11-14
---

# A refusal carries its code

## Why

On 2026-11-13 and 2026-11-14 the family settled its failure contract:

- **The exit contract** (pimalaya-cli a-refused-line-exits-two, eb37962): a command line the grammar refuses exits 2 and, under `--json`, prints the failure object coded `usage-invalid`; a partial outcome exits 3 and above. Himalaya, neverest, cardamum and calendula adopted it. Ortie did not: src/main.rs:110 calls clap's own `Cli::parse()`, so a refused line under `--json` prints clap's text on stderr and nothing on stdout.
- **Codes for refusals** (Himalaya's a-result-carries-its-errors, neverest's a-refusal-carries-its-code, both 2026-11-14): a server's refusal carries a stable code, `permission-denied`, `quota-exceeded`, `rate-limited` and the others of the shared vocabulary, so a caller acts without parsing the message. Ortie has no code at all: every failure prints `"code": null` (`NoErrorCode` at src/cli.rs:17, :103, :106, src/repl.rs:23, :107, src/mcp.rs:44, :429, :598, :633, :705, :904; `ErrorReport::eval` at src/main.rs:135).

An OAuth client's failures are precisely the ones a caller must tell apart: an expired or revoked refresh token (`invalid_grant`) asks the user to run `auth get` again, a refused client secret or certificate (`invalid_client`) asks for a configuration fix, a denied consent (`access_denied`) asks nothing. Today they are flattened to strings where they are built (`anyhow!("<Action> error (code {:?})", res.error)`), naming the code in Rust's Debug spelling (`InvalidGrant`, not the wire `invalid_grant`): src/auth/get.rs:456, :489, :653-682 (`client_credentials_error`, from get.rs:515 and src/token/refresh.rs:91), src/token/refresh.rs:139, src/auth/resume.rs:100-107, :151, src/token/revoke.rs:64, src/mcp.rs:732. io-oauth's `Oauth20AccessTokenErrorParams` is no `std::error::Error`, so no chain walk can find them afterwards: they are coded where they are built. (The paths above are the fact sheet's; the agent confirms each before editing.)

Ortie has no command listing or acting on several items, so the `errors` lists and pimalaya-cli's `ItemError` do not apply to it.

The libraries move first: io-oauth 0.7 moved to pimalaya-stream 0.4 on 2026-11-13, and pimalaya-cli's eb37962 (pinned at 0be1b1f, as Himalaya and neverest pin it) carries `clap::parse`, `usage_error` and `EXIT_USAGE`, which pimalaya-cli 0.4.0 lacks.

## What

Three phases, each one commit, in this order. None needs anything outside this repository: ortie uses no io-pim-discovery, io-webdav, io-gcal nor io-gpeople.

### Phase 1, the libraries

- Cargo.toml: `io-oauth` 0.7, `pimalaya-stream` 0.4 (both or neither: io-oauth 0.7's `Oauth20ClientStd::connect` takes stream 0.4's `Tls`); `[patch.crates-io]` `pimalaya-cli` 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd (no comment; until pimalaya-cli 0.4.1); `cargo update`, which takes pimalaya-config 0.4.1 (a credential command prompting for a password, `client-secret.command` behind a pinentry, reads the terminal again).
- src/config.rs:261-265: the `Tls` built from the `tls` option fills `server: None, client: None`. Ortie has no `tls.cert` option and gains none: its `tls` is a provider name (spec config.md, "One TLS conversion, taking the ALPN list"), and mutual TLS client authentication (RFC 8705) is not something io-oauth implements.
- io-oauth 0.6 to 0.7 and io-http 0.7 to 0.8 change no source, only their dependencies.
- The pin also brings pimalaya-cli's other unreleased changes: the help footer's wording and a manual page per command at every depth. Tests pinning either are updated to match.
- `cargo tree -d --all-features -e normal`: one pimalaya-stream, io-http and io-oauth.
- CHANGELOG: "Bumped io-oauth to 0.6." becomes 0.7 with pimalaya-stream 0.4; a Fixed line for the prompting secret command.

### Phase 2, a refused command line prints its failure object

- src/main.rs:110: `pimalaya_cli::clap::parse::<Cli>()`. A refused line still exits 2; with a `--json` word before any `--` it prints on stdout `{"code":"usage-invalid","error":…,"sources":[],"backtrace":null}`, clap's message without `error: ` nor styles; without it, clap's rendering on stderr as before. Help and version print and exit 0 as clap does.
- REPL: a piped refused line is coded `usage-invalid` by pimalaya-cli itself and loses its `Error: error:` prefix (spec repl.md, "Interactive versus piped output"). The session-flag refusal (`Cannot set -a inside a session`) stays uncoded, pimalaya-cli's.
- MCP: an undeclared argument is refused by pimalaya-cli, coded `usage-invalid` at no cost. Ortie's own argument refusals (src/mcp.rs:781-782, a missing or non-string argument) are coded `usage-invalid` too, a `CodedError::new(CommonCode::UsageInvalid, message)` (`CommonCode` is public at pimalaya-cli 0be1b1f, src/error.rs:52).
- Ortie returns no partial outcome today, so pimalaya-cli's renumbering of `EXIT_PARTIAL` to 3 changes nothing here.
- tests/exit.rs (new, on the built binary, as himalaya/tests/exit.rs and neverest/tests/usage.rs do): `--json token show --nope` prints the coded object and exits 2; the same without `--json` prints on stderr only.

### Phase 3, the codes

- src/error.rs (new): `ErrorCode`, kebab-case, `Serialize` and `JsonSchema`, and the chain walk `error::code(err)`: `code_of(err)` else the first cause io-oauth's typed client error names (`Oauth20ClientStdError`, its `Status(u16)` variants: src/rfc6749/refresh_access_token.rs:86, access_token_request.rs:160, client_credentials.rs:132, rfc8628 token.rs:88 and auth.rs:102, rfc7009 revoke.rs:86, rfc7523 auth_grant.rs:145 and client_auth.rs:162 in io-oauth).
- The vocabulary (a proposal for the owner, see Decisions):

| Code | When |
|---|---|
| `permission-denied` | the grant was refused to the user or the client: `access_denied` at the authorization redirect or in the device flow, Microsoft Entra's `authorization_declined`, a bare HTTP 403 |
| `rate-limited` | a bare HTTP 429; ortie retries nothing (`slow_down` is waited out inside the device flow and never surfaces) |
| `grant-invalid` | `invalid_grant`: the authorization code, the refresh token or the device code was refused, has expired or was revoked; run `auth get` again |
| `client-invalid` | `invalid_client`: the client secret, certificate or assertion was refused; the certificate renewal hint (spec auth.md) stays in the message |
| `client-unauthorized` | `unauthorized_client`: the client may not use this grant |
| `scope-invalid` | `invalid_scope` |
| `device-code-expired` | the device code expired before the user approved it: `expired_token` from the server, or the local deadline (`Oauth20ClientStdError::DeviceCodeExpired`) |
| `state-mismatch` | the authorization redirect carries no `state`, or another one than the request sent (src/auth/resume.rs:109-115) |
| `token-missing` | the account stores no token, or no refresh token where a refresh needs one (src/token/refresh.rs:74, src/account.rs:185, :238-242); run `auth get` |
| `endpoint-missing` | the account configures no endpoint the grant needs: token, device authorization or revocation (src/auth/get.rs:205, :434, :437, :533, src/auth/resume.rs:86, :177, src/token/refresh.rs:109, src/token/revoke.rs:32) |
| `usage-invalid` | pimalaya-cli's, Phase 2 |

- Every other OAuth error code (`invalid_request`, `unsupported_grant_type`, `unsupported_token_type`, `invalid_target`, `server_error`, `temporarily_unavailable`, Entra's `bad_verification_code`, an unknown one) carries no code, the server's word kept in the message.
- Each construction site builds a `CodedError` with its code, and its message names the OAuth code as the server spelled it (`invalid_grant`, through serde, never Debug), followed by `error_description` and `error_uri` as today. The tests matching the Debug spelling (tests/google.rs:55, :67, tests/microsoft.rs:67, tests/token_revoke.rs:145) follow.
- Every failure path reads the vocabulary: `ErrorReport::<ErrorCode>::eval_coded` in src/main.rs:135, `exit_on_failure::<ErrorCode, _>` in src/cli.rs, `repl.run::<_, ErrorCode>` in src/repl.rs:107, `ToolResult::printed::<ErrorCode>`, `error_object` and `ToolResult::error` in src/mcp.rs, and MCP `auth_status`'s nested `error` (src/mcp.rs:843-845) printing the failure object with its code.
- `json-schema`: `ortie-error`'s `code` is typed by `ErrorCode` and `usage-invalid` (src/json_schema.rs:45).
- Tests, through the scripted token endpoints (tests/client_credentials.rs:40 and tests/token_revoke.rs:23 answer any status and body; tests/device_grant.rs:35 scripts the device flow): `invalid_grant` on a refresh (none covers it today), `invalid_client`, `access_denied` in the device flow, a bare 403 and a bare 429, a device code past its deadline, `token show` on an account storing none, each asserting its `code` under `--json` and exit 1.

### Docs

CHANGELOG Unreleased (`**BREAKING**:` as ortie writes it): Added the codes; **BREAKING**: a refused command line under `--json` prints the failure object on stdout; Changed: an OAuth refusal names its code as the server spelled it. README: an exit status line (0, 1 with the failure object, 2 a refused line). No MIGRATION.md is created: ortie has none, and the 3.0.0 CHANGELOG carries the breaking entries. No version bump (3.0.0 waits for the owner's family release).

## Decisions

- **Same id as neverest's change**: ortie, like neverest, codes refusals without `errors` lists.
- **Ortie's own codes beside the shared ones**: the shared vocabulary has nothing for an OAuth grant or client refusal, which is ortie's core domain (neverest left a credential refusal uncoded for want of one). Each code names what the caller does next. These are proposals for the owner to confirm before Phase 3.
- **Coded where built**: io-oauth's OAuth error bodies are no `std::error::Error`, so the chain walk only reads the bare statuses; the bodies are coded at their construction sites.
- **The wire spelling in messages**: `invalid_grant` reads as RFC 6749 §5.2 and the provider's docs write it; `InvalidGrant` was an accident of `{:?}`.
- **No `tls.client`**: Himalaya added mutual TLS to its servers' connections (fa1ea661); an OAuth token endpoint asking for a client certificate is RFC 8705, which io-oauth does not implement, and `tls` stays a provider name.
- **No item errors**: no ortie command lists or acts on several items.

## Scope / non-goals

- io-oauth's `Unknown` error code drops the server's word (Google's `invalid_token` on a revoke, tests/google.rs:64): keeping it is an io-oauth change.
- io-oauth drops the HTTP status when an OAuth body parses, so a 429 carrying a JSON `error` reads by its body alone: also io-oauth's.
- The issuer check of the authorization response (src/auth/resume.rs:117) stays uncoded.
- No version bump; no release.

## Order and parallel work

This change runs in parallel with cardamum's, calendula's (each their a-result-carries-its-errors) and carillon's: none touches this repository, and this one needs no shared prerequisite.
