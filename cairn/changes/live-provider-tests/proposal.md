---
cairn: change
id: live-provider-tests
status: active
created: 2026-11-04
---

# Live provider tests (Google, Microsoft, Fastmail)

## Why

Ortie's four integration suites (tests/client_credentials.rs, device_grant.rs, token_revoke.rs, repl.rs) all run the binary against local mock servers. No test runs the binary against a real provider, so a config that parses and passes the mocks can still fail at Google, Microsoft or Fastmail (wrong endpoint, a refused extra, an error code the hint logic does not expect). The protocol layer gets its own live suites in io-oauth's `live-provider-tests` change. This change covers only what Ortie adds on top: config to grant dispatch, secret commands, storage, auto-refresh, exit codes and JSON output.

## What

Same shape as io-oauth's suites: tests/google.rs, tests/microsoft.rs, tests/fastmail.rs, each test `#[ignore = "requires ..."]`, an `anonymous::` module (network only, every push, forks included) and a `credentialed::` module (org secrets, repository-guarded job), env vars prefixed `ORTIE_`. The existing helpers (config writer, `ortie()` runner, TempDir storage through `cat` and `tee`) move to a shared tests/common.rs. Secrets reach the binary through `client-secret.command = ["printenv", "ORTIE_MICROSOFT_CLIENT_SECRET"]`, never through argv or the config file.

### Microsoft (tests/microsoft.rs)

`anonymous::` (Pimalaya public client `ba9b19e1-973e-4d7c-aed9-848bd2fee385`)

- `device_auth_get_json`: a `grant = "device"` account on the real `/common` endpoints. `ortie --json auth get` succeeds and prints `deviceCode`, `userCode` and `verificationUri`, and storage stays empty. `auth resume` is left out on purpose: it polls until the code expires (15 minutes), and io-oauth already covers the single pending poll.
- `client_credentials_rejected`: a bogus secret gives the error exit code (1, per the output contract), and the error chain names `invalid_client`.

`credentialed::` (org secrets `MICROSOFT_*`)

- `client_credentials_token_show`: from empty storage, `token show --auto-refresh` acquires a token and stores it. A second call serves the same token without a request. With an expired token seeded in storage, a new token is acquired and stored.
- `client_credentials_jwt_token_show`: the same with `grant = "client-credentials-jwt"`, `client-key` and `client-certificate` written to 0600 files in the TempDir. This depends on the certificate secrets requested in io-oauth's change.
- `client_credentials_jwt_renewal_hint`: an unregistered key, or the certificate of a different key, gives `invalid_client` and the certificate renewal hint. This is the only live check of the hint's trigger.

### Google (tests/google.rs)

`anonymous::` only: Ortie has no RFC 7523 section 2.1 grant (deferred in the 2026-08-06 headless-grants log), so it has no headless way to get a Google token.

- `auth_resume_rejected`: an authorization code account on the current Google endpoints (provider-quirks), made-up client id, `auth resume` with a bogus code: error exit code, `invalid_client` in the chain.
- `token_revoke_rejected`: bogus stored token, real revocation endpoint: error exit code, the error carries Google's code, and storage is kept.

If the RFC 7523 section 2.1 grant lands later, a `credentialed::` service account test follows it.

### Fastmail (tests/fastmail.rs)

`anonymous::` only, the account is never involved.

- `auth_resume_rejected`: an account on the endpoints from Fastmail's metadata (token `https://api.fastmail.com/oauth/refresh`), bogus code: error exit code, with the `error_description` surfaced, since it is the only way to tell Fastmail failures apart.

The wizard and its registration left Ortie on 2026-11-08 (samples-replace-the-wizard); the provider sections of config.sample.toml can serve as the accounts these tests write, so a provider drifting from a sample fails here.

### CI

One job per provider in .github/workflows/tests.yml, `cargo test --test <provider> -- --ignored anonymous::` and, guarded by `if: github.repository == 'pimalaya/ortie'`, `credentialed::`. `RUST_LOG=debug` at most: the binary runs io-http, whose trace logs carry secrets and tokens.

## Out of scope

- The authorization code grant through a browser, and refresh with a human-seeded token: io-oauth's manual tier covers the protocol, and Ortie's refresh branch is already covered by mocks.
- MCP server and REPL against live providers: the REPL shares the command paths tested here.

## Open questions

- Two of io-oauth's change: Microsoft certificate secrets, and land the io-oauth suites first so a failure here points at Ortie. Its third, Fastmail registration through the wizard, no longer applies here.
