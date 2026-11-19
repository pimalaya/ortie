---
cairn: delta
change: the-flow-hardened
---

## MODIFIED Requirements
### Requirement: Authorization code grant
When it binds an ephemeral port to build the redirect URI, the same listener SHALL wait for the redirect, so no other process can take the port in between. An interactive `auth get` finishing the flow itself SHALL NOT print the state nor the PKCE verifier, which only a handoff needs.

### Requirement: Manual resume fallback
The human handoff SHALL print the `auth listen` command when the redirect URI is an http one on a loopback host. `--pkce` SHALL refuse a bad verifier without echoing it, clap's refusal included. The `auth resume` command the device grant prints when it hands off SHALL put `--` before the quoted device code.

### Requirement: Loopback listen
`auth listen` SHALL be refused on a redirect URI that is not an http one on a loopback host: the listener speaks plain HTTP, so an https redirect, loopback or not, is finished by hand through `auth resume`.

### Requirement: The flow lives in the server
On an http loopback redirection the server binds the listener. A URI that does not parse, or does not echo the state of the pending flow (coded `state-mismatch`), SHALL be refused by `auth_resume` before the flow is taken, the flow staying pending for the right URI.

### Requirement: Device grant runs
The text output SHALL defuse the user code and the verification URIs with pimalaya-cli's `sanitize::text`.
