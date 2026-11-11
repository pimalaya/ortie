---
cairn: spec
capability: mcp
status: current
---

# MCP server

`ortie mcp` serves one account's OAuth flows to Model Context Protocol clients over stdio. A flow runs in the server: its state, PKCE verifier, redirect URI and device code never leave it, and tools answer receipts and metadata, never a token. Applications keep reading the token through `ortie token show`.

### Requirement: An MCP server over stdio
`ortie mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC message per line, with stdout carrying protocol messages only. `initialize` SHALL answer the client's protocol version when the server speaks it, 2025-06-18 otherwise, and SHALL carry instructions naming the order of the tools. A message that is not JSON SHALL answer -32700; one that is not a JSON object, a batch included, or whose `jsonrpc` is not `"2.0"` -32600; an unknown method -32601. The account SHALL be resolved at start, and its stored token read at most once for the server's lifetime, as in the REPL. A failure resolving the account, or one ending the server, SHALL be printed on stderr with exit status 1, never on stdout; a person missing a configuration meets the welcome instead.

### Requirement: Tools never carry a secret
No tool input or output SHALL carry the access token, the refresh token, the PKCE verifier, the device code or the client secret. `token show` SHALL NOT be exposed.

### Requirement: The flow lives in the server
`auth_get` SHALL start the account's grant, replacing a pending flow and stopping its listener, and answer the authorization URI with `redirectCaptured`, or the user code and verification URI, or on a client credentials account the receipt of the token issued at once. On a loopback redirection, configured or generated on `127.0.0.1`, the server SHALL bind the listener before answering, and SHALL exchange the code and write storage as soon as the redirect arrives. A device flow SHALL be polled at the server's interval, 5 seconds more at each `slow_down`, with the same ending. `auth_status` SHALL report the flow as none, pending, issued with the receipt, or failed with the `ortie-error` object. `auth_resume` SHALL complete the pending code flow from a redirected URI, for a redirection no listener catches.

#### Scenario: A redirect caught by the server
- GIVEN an authorization code account without `endpoints.redirection`
- WHEN `auth_get` is called and the browser follows its redirect to the loopback listener
- THEN the code is exchanged with the server's state, verifier and redirect URI, the token is stored, `auth_status` answers issued, and no response carried the token or the verifier

### Requirement: Revocation is opt-in
`token_inspect`, `auth_get`, `auth_status`, `auth_resume` and `token_refresh` SHALL be listed by default; `token_revoke` only under `--allow-revoke`. Every tool but revocation only replaces the stored token with a newer valid one, and a grant needs the user's consent.

### Requirement: A tool's output is the command's payload
A successful call SHALL return its JSON value as `structuredContent` and as text, described as `outputSchema` by the json-schema registry entry of the command it runs, or by the schema of the `auth_get` and `auth_status` answers, each derived as printed (the serialize contract): a key the answer always prints is required, `null` or not. A failed call SHALL return `isError: true` with the `ortie-error` object. An argument the tool does not declare SHALL be refused naming the declared ones, and a missing or non-string one naming it; an unknown or unlisted tool SHALL be a JSON-RPC error (-32602).

### Requirement: Tool annotations
Each MCP tool SHALL carry `readOnlyHint`, `destructiveHint` and `openWorldHint`. `openWorldHint` SHALL be true on the tools reaching the authorization server (`auth_get`, `auth_resume`, `token_refresh`, `token_revoke`) and false on `token_inspect`, which reads the account's storage only, and `auth_status`, which reports the server's own flow. `token_inspect` and `auth_status` alone SHALL be read-only, and `token_revoke` alone destructive.
