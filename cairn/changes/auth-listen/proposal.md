---
cairn: change
id: auth-listen
status: landed
created: 2026-11-04
---

# `auth listen`: the loopback listener as a command

## Why

Outside a terminal the authorization code flow is a chain of commands: `auth get` hands off the consent URL with its state and PKCE verifier, the user consents, `auth resume` exchanges the code. One link is missing. Only an interactive `auth get` runs the loopback listener (src/auth/get.rs, `await_redirect`), so a script or an agent driving ortie leaves the browser on a dead `127.0.0.1` port, and the user has to copy the redirected URL out of the address bar and hand it back, the code with it. The listener is the step that closes the flow, and it has no command of its own.

The handoff also loses the redirect URI. Without `endpoints.redirection`, `auth get` binds `127.0.0.1:0` and puts the port it got in the authorization URL, but neither the `--json` object nor the printed resume command carries that URI (checked 2026-11-04: the URL said `127.0.0.1:43265`, the fields and the command did not). `auth resume` then exchanges the code without `redirect_uri`, which RFC 6749 §4.1.3 requires whenever the authorization request carried one.

This is also what an MCP server needs (`auth_get` starting the listener and returning only the URL), so the CLI gets the piece first.

## What

1. **`auth listen`.** Authorization code accounts only, refused on the device and client credentials grants as `auth resume`'s code flags are. It takes `--state`, `--pkce` and `--redirect-uri` as `auth resume` does, binds the loopback redirect URI (given, else `endpoints.redirection`), waits for one redirect, then completes exactly as `auth resume`: state check, code exchange, storage write, on-issue hooks, the `{expiresIn, withRefreshToken}` receipt. A redirect URI that is not loopback http(s) with a port is refused, since there is nothing to bind.
2. **The handoff carries the redirect URI.** The `auth get` handoff object gains `redirectUri`, and every printed command carries `--redirect-uri=`, single quoted like the other values.
3. **The human handoff names both ways.** A loopback handoff prints the `auth listen` command, to start before consenting, then the `auth resume` command, for when the listener could not run.
4. **In the REPL.** `auth listen` runs there too, blocking the session until the redirect arrives.

A separate command rather than `auth resume` with an optional positional: resume means "here is the result", listen means "go and get it", and a resume whose missing URI silently starts a server is a surprise.

## Limits

- The listener must be up before the user consents; a redirect that arrives first fails in the browser, and `auth resume` with the copied URL still finishes the flow.
- `auth get` releases the port it picked, so another process can take it before `auth listen` binds; the bind error says so, and the generated URI can be replaced by setting `endpoints.redirection`.
- The listener is single-shot (io-oauth `await_redirect`): whatever request reaches it first is taken as the redirect. A browser sends the redirect first, so this is only a stray-connection risk.
- No timeout: the caller ends a listener it no longer needs.

## Not in scope

The MCP server itself (a change of its own, over this command and the REPL). A timeout flag. Moving the listener into a thread of `auth get`.

## As landed (2026-11-04)

- A loopback redirect URI is refused only when it is not http(s) on a loopback host; a missing port takes the scheme's default, as `auth get`'s own listener does.
- `auth listen` prints "Wait for redirection on …" only to a person at a terminal; piped or under `--json` it prints the receipt alone. `ortie-auth-listen` joins `json-schema`, the receipt's schema.
- The printed commands start with `invocation` (src/auth/get.rs, formerly `resume_invocation`), the `ortie -c … -a …` prefix both commands share.
- ortie's MCP server (cairn/changes/mcp-server) landed first and needs none of this: it holds the redirect URI and binds before answering.
