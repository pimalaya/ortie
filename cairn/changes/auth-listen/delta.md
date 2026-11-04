---
cairn: delta
change: auth-listen
---

## ADDED Requirements

### Requirement: Loopback listen
`auth listen` SHALL, on an authorization-code account, bind the loopback redirect URI given by `--redirect-uri` or `endpoints.redirection`, wait for one redirect, then complete the flow as `auth resume` does with that redirected URI and its `--state` and `--pkce`. It SHALL be refused on the device and client credentials grants, and on a redirect URI that is not loopback http(s) with a port.

## MODIFIED Requirements

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state, PKCE and redirect URI included) after opening the browser. A non-interactive or `--json` `auth get` SHALL hand off with the authorization URI, the state, the PKCE verifier and the redirect URI (`redirectUri`), and its human output SHALL print the `auth listen` command, when the redirect URI is loopback, before the `auth resume` one. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account.

The printed commands SHALL be runnable as printed, whatever the generated values look like: each value SHALL be attached to its flag with `=` and single quoted. A PKCE verifier is drawn from the RFC 7636 unreserved set and a state from URL-safe base64, so either can begin with `-` and be taken for a flag, which the `=` form settles before the parser sees it, or with `~`, which the quotes stop the shell expanding.

The printed commands, and the one the device grant prints when it hands off, SHALL carry the `-c` paths (joined by `:`) and the `-a` name `auth get` ran with, each single quoted, and omit either flag when it was not given. Resumed against another account, the code reaches a token endpoint that did not issue it, or the token lands in another account's storage.
