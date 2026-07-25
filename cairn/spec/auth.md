---
cairn: spec
capability: auth
status: current
---

# Auth

The `auth` command tree obtains tokens by running the grant configured on the account: `discover` runs the wizard (see the discovery capability), `get` initiates the grant, `resume` finishes a flow that could not complete automatically. `get` and `resume` dispatch on the account's configured grant.

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), open it, and capture the redirect on an ephemeral `127.0.0.1` loopback listener, then exchange the code, write storage, and fire the on-issue hooks.

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state and PKCE included) after opening the browser. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account.

### Requirement: Redirection resolution
When `endpoints.redirection` is set it SHALL be used verbatim; otherwise Ortie binds `127.0.0.1:0` and uses the resulting `http://127.0.0.1:<port>` URL as an exact-match loopback redirect (the permitted variable-port exception).

### Requirement: Device grant parses but does not run
`grant = "device"` SHALL parse, but `auth get` and `auth resume` currently bail on a device account. Making the device grant runnable is tracked as an in-flight change.
