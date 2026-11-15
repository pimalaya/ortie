---
cairn: delta
change: mcp-gates
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: Gates list what a tool may do
(mcp) Started without a flag, the server SHALL list only the tools that read, `token_inspect` and `auth_status`. `--allow-write` SHALL list as well the tools replacing the stored token, `auth_get`, `auth_resume` and `token_refresh`, and `--allow-revoke`, which SHALL require `--allow-write`, the revocation `token_revoke`, which leaves the account without a working token until the next grant. A tool a gate leaves out SHALL be refused as any tool not listed. Every tool but revocation only ever replaces the stored token with a newer valid one, and a grant needs the user's consent.

#### Scenario: No gate
- GIVEN an account
- WHEN `ortie mcp` starts without a flag and is asked `tools/list`, then to call `auth_get`
- THEN it lists `token_inspect` and `auth_status` alone, and answers the call -32602

## MODIFIED Requirements

### Requirement: An MCP server over stdio
(mcp) `ortie mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC message per line, with stdout carrying protocol messages only. `initialize` SHALL answer the client's protocol version when the server speaks it, 2025-06-18 otherwise, and SHALL carry instructions naming the order of the tools, the gates listing them and the `ortie-error` object a failed call answers. A message that is not JSON SHALL answer -32700; one that is not a JSON object, a batch included, or whose `jsonrpc` is not `"2.0"` -32600; an unknown method -32601. The account SHALL be resolved at start, and its stored token read at most once for the server's lifetime, as in the REPL. A failure resolving the account, or one ending the server, SHALL be printed on stderr with exit status 1, never on stdout; a person missing a configuration meets the welcome instead.

### Requirement: A tool's output is the command's payload
(mcp) A successful call SHALL return its JSON value as `structuredContent` and as text, described as `outputSchema` by the json-schema registry entry of the command it runs, or by the schema of the `auth_get` and `auth_status` answers, each derived as printed (the serialize contract): a key the answer always prints is required, `null` or not. A failed call SHALL return `isError: true` with the `ortie-error` object, its `code` from the vocabulary. The input schemas SHALL be written by hand, no tool running a clap command. An argument the tool does not declare SHALL be refused naming the declared ones, a missing or `null` one as ``Missing argument `<name>` `` and a non-string one as ``Argument `<name>` must be a string``, the words of pimalaya-cli's grammar, each coded `usage-invalid`; an unknown or unlisted tool SHALL be a JSON-RPC error (-32602).

### Requirement: Tool annotations
(mcp) Each MCP tool SHALL carry `readOnlyHint`, `destructiveHint` and `openWorldHint`, the same whichever gates are open. `openWorldHint` SHALL be false on every tool, none acting beyond the account's own token: reaching the authorization server is reaching the network, which the hint does not mean (pimalaya-cli cairn/spec/mcp.md). `token_inspect` and `auth_status` alone SHALL be read-only, and `token_revoke` alone destructive.

## REMOVED Requirements

### Requirement: Revocation is opt-in
(mcp) Replaced by "Gates list what a tool may do".
