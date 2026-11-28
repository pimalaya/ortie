---
cairn: spec
capability: mcp
status: current
---

# MCP server

`ortie mcp` serves one account's OAuth flows to Model Context Protocol clients over stdio. A flow runs in the server: its state, PKCE verifier, redirect URI and device code never leave it, and tools answer receipts and metadata, never a token. Applications keep reading the token through `ortie token show`.

### Requirement: An MCP server over stdio
`ortie mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC message per line, with stdout carrying protocol messages only. `initialize` SHALL answer the client's protocol version when the server speaks it, 2025-06-18 otherwise, and SHALL carry instructions naming the order of the tools, the gates listing them and the `ortie-error` object a failed call answers. A message that is not JSON SHALL answer -32700; one that is not a JSON object, a batch included, or whose `jsonrpc` is not `"2.0"` -32600; an unknown method -32601. The account SHALL be resolved at start, and its stored token read at most once for the server's lifetime, as in the REPL, save the one re-read a refused refresh makes (see the token capability). A failure resolving the account, or one ending the server, SHALL be printed on stderr with exit status 1, never on stdout; a person missing a configuration meets the welcome instead.

### Requirement: Tools never carry a secret
No tool input or output SHALL carry the access token, the refresh token, the PKCE verifier, the device code or the client secret. `token show` SHALL NOT be exposed.

### Requirement: The flow lives in the server
`auth_get` SHALL start the account's grant, replacing a pending flow and stopping its listener, and answer the authorization URI with `redirectCaptured`, or the user code and verification URI, or on a client credentials account the receipt of the token issued at once. On an http loopback redirection, configured or generated on `127.0.0.1`, the server SHALL bind the listener before answering, and SHALL exchange the code and write storage as soon as the redirect arrives, a request that does not echo the flow's state being skipped (see the auth capability). A replaced flow's listener SHALL be stopped and waited for, up to 2 seconds, so a configured port is free when the next flow binds it. A device flow SHALL be polled at the server's interval, kept between 1 and 60 seconds, 5 seconds more at each `slow_down`, with the same ending, and SHALL fail as expired once its lifetime, capped at one hour, has passed. `auth_status` SHALL report the flow as none, pending, issued with the receipt, or failed with the `ortie-error` object. `auth_resume` SHALL complete the pending code flow from a redirected URI, for a redirection no listener catches. A URI that does not parse (coded `usage-invalid`), or does not echo the state of the pending flow (coded `state-mismatch`), SHALL be refused before the flow is taken, the flow staying pending for the right URI; a URI carrying the server's refusal, or a code the token endpoint refuses, ends the flow as failed.

#### Scenario: A redirect caught by the server
- GIVEN an authorization code account without `endpoints.redirection`
- WHEN `auth_get` is called and the browser follows its redirect to the loopback listener
- THEN the code is exchanged with the server's state, verifier and redirect URI, the token is stored, `auth_status` answers issued, and no response carried the token or the verifier

### Requirement: A tool's output is the command's payload
A successful call SHALL return its JSON value as `structuredContent` and as text, described as `outputSchema` by the json-schema registry entry of the command it runs, or by the schema of the `auth_get` and `auth_status` answers, each derived as printed (the serialize contract): a key the answer always prints is required, `null` or not. A failed call SHALL return `isError: true` with the `ortie-error` object, its `code` from the vocabulary. The input schemas SHALL be written by hand, no tool running a clap command. An argument the tool does not declare SHALL be refused naming the declared ones, a missing or `null` one as ``Missing argument `<name>` `` and a non-string one as ``Argument `<name>` must be a string``, the words of pimalaya-cli's grammar, each coded `usage-invalid`; an unknown or unlisted tool SHALL be a JSON-RPC error (-32602).

### Requirement: Tool annotations
Each MCP tool SHALL carry `readOnlyHint`, `destructiveHint` and `openWorldHint`, the same whichever gates are open. `openWorldHint` SHALL be false on every tool, none acting beyond the account's own token: reaching the authorization server is reaching the network, which the hint does not mean (pimalaya-cli cairn/spec/mcp.md). `token_inspect` and `auth_status` alone SHALL be read-only, and `token_revoke` alone destructive.

### Requirement: Gates list what a tool may do
Started without a flag, the server SHALL list only the tools that read, `token_inspect` and `auth_status`. `--allow-write` SHALL list as well the tools replacing the stored token, `auth_get`, `auth_resume` and `token_refresh`, and `--allow-revoke`, which SHALL require `--allow-write`, the revocation `token_revoke`, which leaves the account without a working token until the next grant. A tool a gate leaves out SHALL be refused as any tool not listed. Every tool but revocation only ever replaces the stored token with a newer valid one, and a grant needs the user's consent.

#### Scenario: No gate
- GIVEN an account
- WHEN `ortie mcp` starts without a flag and is asked `tools/list`, then to call `auth_get`
- THEN it lists `token_inspect` and `auth_status` alone, and answers the call -32602

### Requirement: The MCP agreement is the server's flag
`ortie mcp` SHALL take `--agree-data-use`, the user's agreement, written in the agent's server configuration, to the disclosure `auth get` shows on Pimalaya's Google application (see the auth capability); no tool argument SHALL give it. Without it, `auth_get` on such an account SHALL answer the failure object coded `agreement-missing`, its message holding the disclosure in one sentence and naming the server flag, before the pending flow is replaced. The `auth_get` answer SHALL carry `disclosure`, that sentence, on such an account, and `null` on any other; the server's instructions SHALL tell the agent to show it to the user with the authorization URI or the user code.
