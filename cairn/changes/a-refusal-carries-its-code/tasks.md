---
cairn: tasks
change: a-refusal-carries-its-code
---

# Tasks

## Working rules

- Read first: this change's proposal.md and delta.md; pimalaya-cli's cli/cairn/log/2026-11-13-a-refused-line-exits-two.md and cli/src/error.rs (`CodedError`, `code_of`, `CommonCode`, `usage_error`, `ErrorReport::eval_coded`); himalaya/cairn/log/2026-11-13-a-refused-line-exits-two.md and himalaya/tests/exit.rs; neverest/cairn/log/2026-11-13-a-refused-line-exits-two.md and neverest/src/error.rs; io-oauth's error types (io-oauth/src/client.rs `Oauth20ClientStdError`, src/rfc6749/issue_access_token.rs `Oauth20AccessTokenErrorCode`, src/rfc6749/auth_response.rs `Oauth20AuthErrorCode`, src/rfc7009 revocation codes).
- Every cargo command runs as `nix develop --command cargo ...`. `cargo fmt` after each phase.
- An agent edits, builds and tests; it runs no git write command. The session owning the repository commits each phase with the message given here, no AI trailer.
- Never adjust production code to fit a test. A regression test must fail against the code before the phase.
- No em dashes; code comments wrap at 80 columns; markdown is never hard-wrapped; no secret or token in a fixture or a log.
- Phase checks, each one: `cargo test --all-features`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo clippy --all-targets --no-default-features --features rustls-ring -- -D warnings`; `cargo deny check`.

## Phase 1, the libraries (commit `build: update pimalaya-stream to 0.4 and io-oauth to 0.7`)

- [ ] Cargo.toml: `io-oauth` 0.7, `pimalaya-stream` 0.4; `[patch.crates-io]` pimalaya-cli 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd (no comment; or pimalaya-cli 0.4.1 from crates.io if released, no pin).
- [ ] `cargo update`; the lock holds pimalaya-config 0.4.1.
- [ ] src/config.rs:261-265: `server: None, client: None`.
- [ ] `cargo tree -d --all-features -e normal`: no pimalaya-stream, io-http or io-oauth twice.
- [ ] Tests pinning the help footer or the manual pages follow pimalaya-cli's wording.
- [ ] CHANGELOG: the io-oauth entry names 0.7 and pimalaya-stream 0.4; Fixed: a prompting credential command no longer hangs (pimalaya-config 0.4.1).
- [ ] Phase checks pass.

## Phase 2, the refused line (commit `feat!: print a refused command line as a failure object under --json`)

- [ ] src/main.rs:110: `pimalaya_cli::clap::parse::<Cli>()`.
- [ ] src/mcp.rs:781-782: ortie's own argument refusals coded `usage-invalid` through `CodedError::new(CommonCode::UsageInvalid, ..)`.
- [ ] tests/exit.rs (new): `--json token show --nope` (stdout one failure object coded `usage-invalid`, stderr empty, exit 2); without `--json` (stderr clap's text, stdout empty, exit 2); `--json` after `--` stays an argument.
- [ ] tests/repl.rs: a piped refused line prints `"code":"usage-invalid"` and no `error: ` prefix (tests/repl.rs:39 checks only a string today).
- [ ] tests/mcp.rs:499: the undeclared argument's failure carries `usage-invalid`.
- [ ] Spec: "A refused command line exits 2" (output), "Interactive versus piped output" (repl), "A tool's output is the command's payload" (mcp) folded.
- [ ] Phase checks pass.

## Phase 3, the codes (commit `feat: code the OAuth refusals`)

- [ ] src/error.rs (new, declared in src/main.rs): `ErrorCode` with the proposal's codes, `error::code` reading `code_of` then io-oauth's bare `Status(403)` and `Status(429)` in the chain.
- [ ] Construction sites coded, each message naming the OAuth code in its wire spelling: src/auth/get.rs:456, :489, :653-682 (`client_credentials_error`, the JWT renewal hint kept), src/token/refresh.rs:139, src/auth/resume.rs:100-107, :109-115, :151, src/token/revoke.rs:64, src/mcp.rs:680 (local device expiry), :732; `token-missing` at src/token/refresh.rs:74 and the storage reads (src/account.rs:185, :238-242) where no token is stored; `endpoint-missing` at src/auth/get.rs:205, :434, :437, :533, src/auth/resume.rs:86, :177, src/token/refresh.rs:109, src/token/revoke.rs:32. Confirm each path and line before editing: the list comes from a read-only survey.
- [ ] Every `NoErrorCode` becomes `ErrorCode` (src/cli.rs:17, :103, :106, src/repl.rs:23, :107, src/mcp.rs:44, :429, :598, :633, :705, :904); src/main.rs:135 `ErrorReport::<ErrorCode>::eval_coded`; `auth_status`'s nested `error` (src/mcp.rs:843-845) carries the code.
- [ ] src/json_schema.rs:45: `ortie-error`'s `code` typed by `ErrorCode` and `usage-invalid`; `every_command_is_described` (src/json_schema.rs:64-75) still passes.
- [ ] Tests: through tests/client_credentials.rs:40, tests/token_revoke.rs:23 and tests/device_grant.rs:35's mock servers, and tests/repl.rs:69's refresh: `invalid_grant` on a refresh, `invalid_client`, device-flow `access_denied`, a bare 403, a bare 429, a device code past its deadline, `token show` with no stored token; each asserts `code` under `--json` and exit 1. tests/google.rs:55, :67, tests/microsoft.rs:67 and tests/token_revoke.rs:145 match the wire spelling.
- [ ] Spec: "A failure carries a stable code" (output), "The JSON contract is described and versioned" (output), "A bare error status is named" (auth) folded.
- [ ] Phase checks pass.

## Landing

- [ ] CHANGELOG (Added, **BREAKING**, Changed, Fixed as in the proposal); README exit statuses.
- [ ] Delta folded into cairn/spec/output.md, auth.md, repl.md and mcp.md; log cairn/log/<date>-a-refusal-carries-its-code.md (frontmatter `cairn: log`, `change`, `landed:`, as ortie's change logs write it) naming what landed per phase, the evidence and the capabilities moved; status landed.
- [ ] The live suites (`--ignored anonymous::` for google, microsoft and fastmail; `credentialed::` upstream) run by the owner on the result.
