---
cairn: delta
id: the-rulings-before-3-0-0
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A stored token answers a receipt (output)
`token refresh`, `auth listen`, `auth resume` and a token-issuing `auth get` print `TokenIssuedOutput`, `{account, expiresIn, withRefreshToken}`, `account` the name of the account the token was stored for. The text reads "Access token of account <name> successfully issued" or "refreshed", with the expiry.

### Requirement: A failure carries a stable code (output)
The `usage-invalid` row no longer names a missing redirection. `endpoint-missing` covers `auth listen` with neither `--redirect-uri` nor `endpoints.redirection`. A new row: `redirection-unsupported`, `auth listen` on an `endpoints.redirection` that is not an http URI on a loopback host.

### Requirement: Loopback listen (auth)
The missing redirection fails coded `endpoint-missing` with exit 1, an `endpoints.redirection` it cannot bind coded `redirection-unsupported` with exit 1; a `--redirect-uri` or a handoff `redirectUri` it cannot bind stays a refused command line, coded `usage-invalid` with exit 2, as the device and client credentials grants do.

### Requirement: Manual resume fallback (auth)
The authorization-code handoff carries the authorization URI, the state, the PKCE verifier and the redirect URI, nothing about the shell `auth get` ran in.

### Requirement: Device grant runs (auth)
The device response printed carries nothing about the shell `auth get` ran in.

### Requirement: The handoff can stay off the command line (auth)
A handoff carrying a key Ortie no longer prints, such as the `interactive` of an earlier build, is read, that key ignored.

### Requirement: JWT assertion credentials (config)
The certificate key is `client-cert`, `client-certificate` read as its alias.

## REMOVED Requirements

None.
