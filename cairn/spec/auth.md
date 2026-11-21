---
cairn: spec
capability: auth
status: current
---

# Auth

The `auth` command tree obtains tokens by running the grant configured on the account: `get` initiates the grant, `listen` catches the redirection of a flow `get` handed off, `resume` finishes a flow that could not complete automatically. Each dispatches on the account's configured grant. An account comes from the configuration the user writes, starting from a sample (see the config capability).

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), print it for the user to open, and capture the redirect on a loopback listener, then exchange the code and write storage. Ortie SHALL NOT launch a browser. When it binds an ephemeral port to build the redirect URI, the same listener SHALL wait for the redirect, so no other process can take the port in between. An interactive `auth get` finishing the flow itself SHALL NOT print the state nor the PKCE verifier, which only a handoff needs.

### Requirement: Only the redirect of the flow ends the wait
A loopback listener, the one of `auth get`, of `auth listen` or of the MCP server, SHALL end its wait only on a `GET` to the path of the redirect URI carrying a `code` or an `error` and echoing the state of the flow, when it holds one (`auth listen` run without `--state` holds none). Any other connection SHALL be answered `404 Not Found` and dropped, after at most 5 seconds and a request line of 8 KiB, and the wait SHALL go on: a stray or forged request to the port cannot end the flow. The page answered to the redirect SHALL NOT claim a success, the code not being exchanged yet.

### Requirement: The issuer is not compared
An account names no issuer and Ortie reads no server metadata, so the `iss` parameter of an authorization response (RFC 9207) SHALL NOT be compared: the state and PKCE bind the response to its request.

### Requirement: A bare error status is named
When the token, device authorization or revocation endpoint answers an error status without an OAuth error body, the command SHALL fail naming the HTTP status, coded `permission-denied` for a 403 and `rate-limited` for a 429.

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state, PKCE and redirect URI included) after the authorization URL. A non-interactive or `--json` `auth get` SHALL hand off with the authorization URI, the state, the PKCE verifier and the redirect URI (`redirectUri`), nothing about the shell it ran in, and its human output SHALL print the `auth listen` command, when the redirect URI is an http one on a loopback host, before the `auth resume` one. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account, one that does not parse refused as a refused command line, coded `usage-invalid` with exit 2, without echoing it.

The printed commands SHALL be runnable as printed, whatever the generated values look like: each value SHALL be attached to its flag with `=` and quoted for the platform's shells, single quoted on Unix and double quoted on Windows, which cmd and PowerShell both read. A PKCE verifier is drawn from the RFC 7636 unreserved set and a state from URL-safe base64, so either can begin with `-` and be taken for a flag, which the `=` form settles before the parser sees it, or with `~`, which the quotes stop the shell expanding. `--pkce` SHALL refuse a verifier that is not 43 to 128 characters of that set, without echoing it, clap's refusal included, and `--state` an empty state or one holding a byte outside printable ASCII.

The `auth resume` command the device grant prints when it hands off SHALL put `--` before the quoted device code, which a server can start with `-`.

The printed commands, and the one the device grant prints when it hands off, SHALL carry one `-c` per configuration path and the `-a` name `auth get` ran with, each quoted the same way, and omit either flag when it was not given. Resumed against another account, the code reaches a token endpoint that did not issue it, or the token lands in another account's storage.

### Requirement: The handoff can stay off the command line
`auth listen` and `auth resume` SHALL take `--handoff <PATH>`, the object a `--json` `auth get` prints, read from that file or, for `-`, from stdin, as the family's other JSON inputs take `-`. It SHALL give the `state`, `pkceCodeVerifier` and `redirectUri` (the last unless `--redirect-uri` is given), its other keys ignored, one an earlier build printed (`interactive`) included, so `ortie --json auth get > flow.json` then `ortie auth listen --handoff flow.json` keeps the flow's secrets off a command line other local users can read. `--handoff` beside `--state` or `--pkce` SHALL be refused as clap refuses a conflict (`usage-invalid`, exit 2). At most 64 KiB SHALL be read; an object that does not parse SHALL fail naming its line and column, never a value. `--handoff` SHALL be refused on any grant but the authorization code one as a refused command line, coded `usage-invalid` with exit 2, and `--handoff -` inside a session, which reads its lines from stdin, through pimalaya-cli's `session::check_stdin`, coded `stdin-taken`. Without `--handoff`, neither command SHALL read stdin. `--state` and `--pkce` stay, their values visible to other local users while the command runs.


### Requirement: An interrupted wait hands off
When a termination signal (SIGINT, SIGTERM, SIGHUP; Ctrl-C or Ctrl-Break on Windows) reaches an interactive `auth get` waiting for the redirection, it SHALL print the `auth resume` command a fallback prints, with the state, the PKCE verifier and the redirect URI, then exit 130, pimalaya-cli's status for a process a signal ended (`signal::EXIT_SIGNALED`). In a REPL session no handler SHALL be installed: the signal ends the session with 130 as it would without Ortie, no handoff printed, and a later Ctrl-C at the prompt is not swallowed.

### Requirement: Redirection resolution
When `endpoints.redirection` is set it SHALL be used verbatim, except an `http` loopback redirection naming no port, which Ortie binds on an ephemeral port of its host and sends with that port (RFC 8252 section 7.3). When it is not set, Ortie binds `127.0.0.1:0` and uses the resulting `http://127.0.0.1:<port>` URL. Both are exact-match loopback redirects under the permitted variable-port exception.

### Requirement: Device authorization endpoint
An account SHALL accept `endpoints.device-authorization`, checked by `auth get` only on a device account.

### Requirement: Device grant runs
On a device account, `auth get` SHALL request device authorization, display the user code and verification URI (preferring `verification_uri_complete` when present), each defused by pimalaya-cli's `sanitize::text` in the text output,, and either poll to completion (interactive) or print the device response and hand off (non-interactive / `--json`), then write storage as the code grant does. `auth resume` SHALL interpret its positional as the device code, and the authorization-code-only flags (`--state`, `--pkce`, `--redirect-uri`) on a device account, or an empty device code, SHALL be refused as a refused command line, coded `usage-invalid` with exit 2. Account `extras` are not forwarded on the device authorization request.

The lifetime and the polling interval the server states SHALL be bounded before use, the lifetime capped at one hour and the interval kept between 1 and 60 seconds, 5 seconds more at each `slow_down`, and no wait SHALL outlast the lifetime: a server cannot make Ortie wait without end, nor poll without pause. The device response printed SHALL carry the bounded values, and nothing about the shell `auth get` ran in. A resume, handed the device code alone, polls every 5 seconds for 30 minutes.

### Requirement: Client credentials grants
The flat `grant` selector SHALL accept `client-credentials` (RFC 6749 section 4.4, client authenticated by `client-secret`) and `client-credentials-jwt` (RFC 7523 section 2.2, client authenticated by a signed JWT assertion). On these accounts `auth get` SHALL run the exchange headlessly in one shot against `endpoints.token` and write storage; `auth resume` SHALL be refused as a refused command line, coded `usage-invalid` with exit 2, since there is nothing to resume.

### Requirement: Certificate renewal hint
When the JWT kind is rejected with `invalid_client`, the reported error SHALL carry a hint that the certificate credential is likely expired and needs renewal.

### Requirement: Loopback listen
`auth listen` SHALL, on an authorization-code account, bind the loopback redirect URI given by `--redirect-uri` or `endpoints.redirection`, wait for one redirect, then complete the flow as `auth resume` does with that redirected URI and its `--state` and `--pkce`. It SHALL be refused as a refused command line, coded `usage-invalid` with exit 2, on the device and client credentials grants, and on a `--redirect-uri`, or a handoff `redirectUri`, that is not an http URI on a loopback host, and with neither `--redirect-uri`, a handoff `redirectUri` nor `endpoints.redirection`, the message pointing at the redirect URI `auth get` printed, since `auth get` bound an ephemeral port a configuration added afterwards cannot name. An `endpoints.redirection` that is not an http URI on a loopback host SHALL fail with exit 1, coded `redirection-unsupported`, the message naming `endpoints.redirection`. The listener speaks plain HTTP, so an https redirect, loopback or not, is finished by hand through `auth resume`.
