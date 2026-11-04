---
cairn: delta
change: mcp-server
---

## ADDED Requirements

### Requirement: An MCP server over stdio
`ortie mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC message per line, with stdout carrying protocol messages only. The account SHALL be resolved at start, and its stored token read at most once for the server's lifetime, as in the REPL.

### Requirement: Tools never carry a secret
No tool input or output SHALL carry the access token, the refresh token, the PKCE verifier, the device code or the client secret. `token show` SHALL NOT be exposed.

### Requirement: The flow lives in the server
`auth_get` SHALL start a flow whose state, verifier, redirect URI or device code stay in the server, and return the consent URL, or the user code and verification URI, or on a client credentials account the receipt. On a loopback redirect the server SHALL bind the listener before returning the consent URL, and SHALL exchange the code, write storage and fire the on-issue hooks as soon as the redirect arrives; a device flow SHALL be polled the same way. `auth_status` SHALL report the flow as none, pending, issued with the receipt, or failed with the error. `auth_resume` SHALL complete the pending flow from a redirected URI. A new `auth_get` SHALL replace a pending flow.

### Requirement: Revocation is opt-in
`token_inspect`, `auth_get`, `auth_status`, `auth_resume` and `token_refresh` SHALL be listed by default; `token_revoke` only under `--allow-revoke`.

### Requirement: A tool's output is the command's payload
A successful call SHALL return its JSON value as `structuredContent` and as text, described by its json-schema registry entry as `outputSchema`. A failed call SHALL return `isError: true` with the `ortie-error` object; an unknown tool or an unknown argument SHALL be refused, the latter naming the known ones.
