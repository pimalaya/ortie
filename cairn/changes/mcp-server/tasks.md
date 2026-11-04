---
cairn: tasks
change: mcp-server
---

# Tasks

## Before (other repositories)

- [ ] pimalaya-cli: `mcp` module with the JSON-RPC loop, the capture printer and the failure mapping (deferred: copied for now, see the proposal)
- [x] io-oauth: accept the redirect on a caller-bound `TcpListener` (`client::await_redirect_on`); release pending, path-patched meanwhile

## ortie

- [x] `Command::Mcp` (src/mcp.rs): account resolved at start, stdin reader thread, one event channel
- [x] Pending flow: state, verifier, redirect URI or device code held in memory, never serialized; a new `auth_get` replaces it and stops its listener
- [x] Code flow: bind, authorization URI, background accept, exchange on arrival
- [x] Device flow: user code and verification URI returned, polling at the server interval, exchange on approval
- [x] Client credentials: `auth_get` completes in the call
- [x] Tools `token_inspect`, `auth_get`, `auth_status`, `auth_resume`, `token_refresh`; `token_revoke` behind `--allow-revoke`
- [x] Hand-written input schemas, `additionalProperties: false`, unknown keys refused naming the known ones
- [x] `outputSchema` from the registry, or from the `auth_get` and `auth_status` types
- [x] Failures as `isError` with `ortie-error`; an unknown tool a JSON-RPC error
- [x] Tests over stdio with a mock authorization server, asserting no secret on stdout
- [x] README: the server and a client configuration; CHANGELOG
- [x] Fold into a new cairn/spec/mcp.md; log; land
