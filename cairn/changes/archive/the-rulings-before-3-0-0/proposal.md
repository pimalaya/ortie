---
cairn: change
id: the-rulings-before-3-0-0
status: landed
created: 2026-11-21
---

# The rulings before 3.0.0

## Why

Four owner decisions of 2026-11-21 settle what the 3.0.0 contract prints before it ships, each being a break once released.

- The token receipt names nothing it acted on, where pimalaya-cli cairn/spec/output.md rules that a write SHALL name its target: a script running `token refresh` over several accounts cannot tell the receipts apart.
- The `auth get` handoffs print `interactive`, which only reports whether stdout is a terminal: under `--json` or a pipe it is always `false`, so it tells a script nothing.
- The 2026-11-21 change the-grant-refusals-exit-2 made every `auth listen` refusal exit 2 coded `usage-invalid`, a missing or unlistenable `endpoints.redirection` included. Exit 2 means a refused command line; a fault of the configuration is a failure (exit 1) with a code naming it. The rule: when changing the command line would fix it, exit 2; when only changing the configuration would, exit 1.
- The configuration key is `client-cert`, matching `tls.cert` and `tls.client.cert`, with `client-certificate` read as an alias; the spec, the tests and a doc comment still wrote `client-certificate`.

## What

- `TokenIssuedOutput` gains `account`, the account name, as its first key: `{account, expiresIn, withRefreshToken}`, printed by `token refresh`, `auth listen`, `auth resume`, a token-issuing `auth get` and the MCP receipts (`auth_get`, `auth_status`, `auth_resume`, `token_refresh`), which share the type. Its text reads `Access token of account <name> successfully issued (expires in <duration>)`, or `refreshed`.
- `interactive` leaves the serialization and the schema of the authorization-code and device handoffs, kept for the text form alone. A handoff file carrying it, written by an earlier build, is still read: `AuthHandoff` ignores the keys it does not take.
- `auth listen`: with neither `--redirect-uri` nor `endpoints.redirection`, exit 1 coded `endpoint-missing`; with an `endpoints.redirection` that is not an http loopback URI, exit 1 coded `redirection-unsupported` (new), the message naming `endpoints.redirection`. A `--redirect-uri`, or a handoff's `redirectUri`, that is not one stays exit 2 coded `usage-invalid`.
- Every other refusal the-grant-refusals-exit-2 moved to exit 2 stays there, each fixed by changing the command line: `auth listen` on a device or client credentials account (run `auth resume` or `auth get`), `auth resume` on a client credentials account (run `auth get`), `--state`, `--pkce` or `--redirect-uri` on a device account (drop them), an empty device code and a redirected URI that does not parse (pass the right one), `--handoff` on another grant (drop it).
- `client-cert` written everywhere Ortie writes the key; the alias stays.
- Kept: `TokenIssuedOutput` as one type; the handoff reader's tolerance of unknown keys; the `--redirect-uri` refusal's wording.
