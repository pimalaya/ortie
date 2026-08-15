---
cairn: spec
capability: auth
status: current
---

# Auth

The `auth` command tree obtains tokens by running the grant configured on the account: `get` initiates the grant, `resume` finishes a flow that could not complete automatically. Both dispatch on the account's configured grant. The wizard that produces an account in the first place is not part of this tree; `ortie configure` runs it (see the discovery capability).

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), open it, and capture the redirect on an ephemeral `127.0.0.1` loopback listener, then exchange the code, write storage, and fire the on-issue hooks.

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state and PKCE included) after opening the browser. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account.

The printed command SHALL be runnable as printed, whatever the generated values look like: each value SHALL be attached to its flag with `=` and single quoted. A PKCE verifier is drawn from the RFC 7636 unreserved set and a state from URL-safe base64, so either can begin with `-` and be taken for a flag, which the `=` form settles before the parser sees it, or with `~`, which the quotes stop the shell expanding.

### Requirement: Redirection resolution
When `endpoints.redirection` is set it SHALL be used verbatim; otherwise Ortie binds `127.0.0.1:0` and uses the resulting `http://127.0.0.1:<port>` URL as an exact-match loopback redirect (the permitted variable-port exception).

### Requirement: Device authorization endpoint
An account SHALL accept `endpoints.device-authorization`, checked by `auth get` only on a device account.

### Requirement: Device grant runs
On a device account, `auth get` SHALL request device authorization, display the user code and verification URI (preferring `verification_uri_complete` when present), and either poll to completion (interactive) or print the device response and hand off (non-interactive / `--json`), then write storage and fire the on-issue hooks shared with the code grant. `auth resume` SHALL interpret its positional as the device code, and the authorization-code-only flags (`--state`, `--pkce`, `--redirect-uri`) SHALL be rejected on device accounts. Account `extras` are not forwarded on the device authorization request.

### Requirement: Client credentials grants
The flat `grant` selector SHALL accept `client-credentials` (RFC 6749 section 4.4, client authenticated by `client-secret`) and `client-credentials-jwt` (RFC 7523 section 2.2, client authenticated by a signed JWT assertion). On these accounts `auth get` SHALL run the exchange headlessly in one shot against `endpoints.token`, write storage and fire the on-issue hooks; `auth resume` SHALL be rejected since there is nothing to resume.

### Requirement: Certificate renewal hint
When the JWT kind is rejected with `invalid_client`, the reported error SHALL carry a hint that the certificate credential is likely expired and needs renewal.
