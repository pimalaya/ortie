---
cairn: change
id: mcp-server
status: landed
created: 2026-11-04
---

# `ortie mcp`: OAuth flows piloted by an agent

## Why

An agent configuring himalaya, cardamum or calendula for a user hits the OAuth step and stops: it can run `ortie auth get`, but the flow then needs a listener, a human consent and a code exchange, and every handoff field (state, PKCE verifier, device code) passes through the agent's context. A server kept alive for one account can hold those values itself, run the listener and the device polling in the background, and give the agent only what it must show the user: a consent URL or a user code. The token never crosses: tools print receipts and metadata, and the applications keep reading the token out of band, through `ortie token show` as their secret command.

himalaya ships the same shape (himalaya cairn/spec/mcp.md): MCP revision 2025-06-18 over stdio, one account per server, outputs as the commands' `--json` payloads, failures as `isError` results carrying the `ortie-error` object. The server is also a kept-alive process, so like the REPL it reads storage once and unlocks a keyring once.

## What

1. **Transport.** `ortie mcp` serves `initialize`, `ping`, `tools/list` and `tools/call` over stdio, one JSON-RPC message per line, stdout carrying protocol messages only, logs on stderr. Bound to the `-a` or default account, resolved at start so a broken configuration fails the server, not its first call. Not available inside the REPL (absent from its tree).
2. **Tools.**

   | Tool | Input | Output | Listed |
   |---|---|---|---|
   | `token_inspect` | none | `ortie-token-inspect` | always |
   | `auth_get` | none | consent URL, or user code and verification URI, or the receipt (client credentials) | always |
   | `auth_status` | none | the pending flow: `none`, `pending`, `issued` with the receipt, `failed` with the error | always |
   | `auth_resume` | `uri`, the redirected URI | the receipt | always |
   | `token_refresh` | none | the receipt | always |
   | `token_revoke` | none | `ortie-token-revoke` | `--allow-revoke` |

   `token show` is never a tool: its output is the bearer token, which would reach the model provider. `configure` is not either: it writes a local file, as himalaya's rule excludes.
3. **The flow lives in the server.** `auth_get` creates a pending flow holding the state, the PKCE verifier and the redirect URI (or the device code), and returns none of them. On a loopback redirect the server binds the listener before building the consent URL, so no other process can take the port, and accepts the redirect on a background thread. The exchange runs as soon as the redirect arrives, without waiting for an `auth_status` call: an authorization code expires within minutes, and the user who consented expects the token stored. A device flow polls at the server's interval on the same terms. One flow at a time: a new `auth_get` replaces the pending one and stops its listener.
4. **`auth_resume` for redirects no listener can catch**, such as a private-use scheme: the user hands the redirected URI to the agent, which passes it on. The code it carries is useless without the verifier, which stays in the server.
5. **Threads.** A stdin reader thread and the listener thread feed one channel; the main thread owns the account and runs every request, exchange and device poll in order, so the account needs no lock and a storage write never races another.
6. **Gate.** Every listed tool except `token_revoke` only ever replaces the stored token with a valid newer one, and `auth_get` needs a human to consent, so they are listed by default. `token_revoke` leaves the account without a working token and is reachable by a prompt injection, so it is opt-in. This departs from himalaya's `--allow-write`, under which an ortie server would list two tools by default; the departure is deliberate and named in the spec.
7. **Schemas.** Inputs are written by hand rather than generated from clap: no tool takes a CLI flag, and the code-flow flags (`--state`, `--pkce`, `--redirect-uri`) are exactly what must not cross. `outputSchema` comes from the json-schema registry; the `auth_get` and `auth_status` shapes get entries of their own (`ortie-mcp-auth-get`, `ortie-mcp-auth-status`).
8. **Shared code.** The JSON-RPC loop, the capture printer and the failure mapping are himalaya's (src/mcp.rs `McpServer::answer`, `McpPrinter`, `failure`), and the clap-to-tools half is not needed here. With ortie a second consumer, they move into pimalaya-cli first (an `mcp` module, a minor release), then both binaries adopt them, rather than a third copy beside the two REPL loops.
9. **io-oauth.** `await_redirect` binds its own listener; the server needs it to accept on a listener it bound earlier, so io-oauth gains a variant taking a `TcpListener`.

## Depends on

- io-oauth's listener variant, released.

## Later

- Telling the client when a flow completes (`notifications/message` under the logging capability), so the agent need not poll `auth_status`.
- URL-mode elicitation, from MCP revisions after 2025-06-18, handing the consent URL to the client's own interface instead of through the model (check which revision carries it and which clients answer it).
- An OAuth discovery tool for an email address, the wizard's search without the prompts and without writing configuration.

## Not in scope

Several accounts per server (several servers in the client's configuration). An HTTP transport. Any tool returning the access or refresh token.

## As landed (2026-11-04)

- The shared code (item 8) is copied from himalaya into src/mcp.rs; moving it into pimalaya-cli is left for when himalaya and ortie both adopt it, so this release waits on no pimalaya-cli release.
- The `auth_get` and `auth_status` schemas (item 7) come from their types (`McpAuthGetOutput`, `McpAuthStatusOutput`), not from the json-schema registry, which keys commands only.
- `auth_get` answers `redirectCaptured`, telling the agent whether to ask the user for the redirected URL; `initialize` carries instructions naming the tool order.
- cairn/changes/auth-listen is not a dependency: the server holds the redirect URI itself.
