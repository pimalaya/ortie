---
cairn: log
change: mcp-server
date: 2026-11-04
---

# `ortie mcp`

Added `ortie mcp` (src/mcp.rs): an MCP server over stdio for one account with the tools `token_inspect`, `auth_get`, `auth_status`, `auth_resume`, `token_refresh`, and `token_revoke` under `--allow-revoke`. The main thread owns the account; a stdin reader thread and the loopback listener thread feed it events, and a pending device flow wakes it at each poll. `auth_get` binds the listener before answering the authorization URI, through io-oauth's new `client::await_redirect_on` (io-oauth cairn/log/2026-11-04-caller-bound-redirect.md, path-patched until released). src/auth/get.rs gained `authorization_request` and `request_device_authorization`, shared by `auth get` and the server.

Departures from the proposal: the JSON-RPC loop and the capture printer are copied from himalaya rather than moved into pimalaya-cli first, so this release waits on no pimalaya-cli release; the `auth_get` and `auth_status` schemas come straight from their types, the json-schema registry keying commands only; cairn/changes/auth-listen turned out not to be needed, the server holding the redirect URI itself.

Capabilities moved: `mcp` (new).

Verified: tests/mcp.rs drives the binary against a mock authorization server, the test playing the browser: listing and the revocation gate, a caught code flow exchanged with the server's verifier and redirect URI, an uncaught redirect resumed from its URI, a device flow polled through `authorization_pending`, a replaced flow whose listener stops, refused arguments and tools; every test asserts that no response carried the access token, the refresh token, the verifier or the device code.
