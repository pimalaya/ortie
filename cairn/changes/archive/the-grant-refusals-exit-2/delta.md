---
cairn: delta
id: the-grant-refusals-exit-2
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Device grant runs (auth)
The authorization-code-only flags on a device account, and an empty device code, are refused as a refused command line, coded `usage-invalid` with exit 2.

### Requirement: Client credentials grants (auth)
`auth resume` on these accounts is refused as a refused command line, coded `usage-invalid` with exit 2.

### Requirement: Loopback listen (auth)
`auth listen` on the device and client credentials grants, and on a redirect URI that is not an http one on a loopback host, is refused as a refused command line, coded `usage-invalid` with exit 2, as the missing redirection is.

### Requirement: Manual resume fallback (auth)
A redirected URI that does not parse is refused as a refused command line, coded `usage-invalid` with exit 2, never echoed.

### Requirement: A failure carries a stable code (output)
The `usage-invalid` row names every refusal Ortie makes after parsing: a command or a flag the account's grant has no use for, a redirection `auth listen` cannot bind, an empty device code, a redirected URI that does not parse.

### Requirement: The flow lives in the server (mcp)
An `auth_resume` URI that does not parse fails coded `usage-invalid`.

## REMOVED Requirements

None.
