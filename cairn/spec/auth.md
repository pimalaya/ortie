---
cairn: spec
capability: auth
status: current
---

# Auth

The `auth` command tree obtains tokens by running the grant configured on the account: `get` initiates the grant, `listen` catches the redirection of a flow `get` handed off, `resume` finishes a flow that could not complete automatically. Each dispatches on the account's configured grant. An account comes from the configuration the user writes, starting from a sample (see the config capability).

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), print it for the user to open, and capture the redirect on a loopback listener, then exchange the code and write storage. Ortie SHALL NOT launch a browser.

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state, PKCE and redirect URI included) after opening the browser. A non-interactive or `--json` `auth get` SHALL hand off with the authorization URI, the state, the PKCE verifier and the redirect URI (`redirectUri`), and its human output SHALL print the `auth listen` command, when the redirect URI is loopback, before the `auth resume` one. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account.

The printed commands SHALL be runnable as printed, whatever the generated values look like: each value SHALL be attached to its flag with `=` and single quoted. A PKCE verifier is drawn from the RFC 7636 unreserved set and a state from URL-safe base64, so either can begin with `-` and be taken for a flag, which the `=` form settles before the parser sees it, or with `~`, which the quotes stop the shell expanding.

The printed commands, and the one the device grant prints when it hands off, SHALL carry one `-c` per configuration path and the `-a` name `auth get` ran with, each single quoted, and omit either flag when it was not given. Resumed against another account, the code reaches a token endpoint that did not issue it, or the token lands in another account's storage.

### Requirement: Redirection resolution
When `endpoints.redirection` is set it SHALL be used verbatim, except an `http` loopback redirection naming no port, which Ortie binds on an ephemeral port of its host and sends with that port (RFC 8252 section 7.3). When it is not set, Ortie binds `127.0.0.1:0` and uses the resulting `http://127.0.0.1:<port>` URL. Both are exact-match loopback redirects under the permitted variable-port exception.

### Requirement: Device authorization endpoint
An account SHALL accept `endpoints.device-authorization`, checked by `auth get` only on a device account.

### Requirement: Device grant runs
On a device account, `auth get` SHALL request device authorization, display the user code and verification URI (preferring `verification_uri_complete` when present), and either poll to completion (interactive) or print the device response and hand off (non-interactive / `--json`), then write storage as the code grant does. `auth resume` SHALL interpret its positional as the device code, and the authorization-code-only flags (`--state`, `--pkce`, `--redirect-uri`) SHALL be rejected on device accounts. Account `extras` are not forwarded on the device authorization request.

### Requirement: Client credentials grants
The flat `grant` selector SHALL accept `client-credentials` (RFC 6749 section 4.4, client authenticated by `client-secret`) and `client-credentials-jwt` (RFC 7523 section 2.2, client authenticated by a signed JWT assertion). On these accounts `auth get` SHALL run the exchange headlessly in one shot against `endpoints.token` and write storage; `auth resume` SHALL be rejected since there is nothing to resume.

### Requirement: Certificate renewal hint
When the JWT kind is rejected with `invalid_client`, the reported error SHALL carry a hint that the certificate credential is likely expired and needs renewal.

### Requirement: Loopback listen
`auth listen` SHALL, on an authorization-code account, bind the loopback redirect URI given by `--redirect-uri` or `endpoints.redirection`, wait for one redirect, then complete the flow as `auth resume` does with that redirected URI and its `--state` and `--pkce`. It SHALL be refused on the device and client credentials grants, and on a redirect URI that is not loopback http(s).
