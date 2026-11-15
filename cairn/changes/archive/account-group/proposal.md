---
cairn: change
id: account-group
status: landed
created: 2026-11-15
---

# The `account` group: `account list` and `account check`

## Why

Every Pimalaya CLI carries an `account` group holding `account list` and `account check` (pimalaya-cli cairn/spec/commands.md, "Every binary carries an account group"); Ortie had none (finding F1 of the CLI alignment round of 2026-11-15, owner rulings D1 and P4). The family's check opens each backend an account declares. Ortie has no backend, so the ruling asks what its check proves; P4 answered with the two things the question offered: the account's storage commands run, and its endpoints answer. This proposal says exactly what is run and sent, for the owner to review.

## What

1. **`ortie account list`** (alias `ls`) prints pimalaya-cli's `AccountListOutput`, one `AccountRow` per account sorted by name, read from the configuration alone: no storage command runs and no request is sent, and it needs no default account. `backends` and `notBuilt` are always empty: they mean the backends a build serves or leaves out, which Ortie has none of, and putting the grant or the endpoints there would mislead a script reading the same key across the family. The text form prints `-` in that column, as every family listing prints an empty cell.
2. **`ortie account check`** takes the account as any account-bound command does and prints pimalaya-cli's `AccountCheckOutput<ErrorCode>` with two rows, in this order, both always run:
   - **`storage`**: `storage.read.command` runs once, as `token show` runs it, and passes when it exits successfully with a token JSON that parses. An empty store fails coded `token-missing`, as `token show` does; a command that cannot spawn, fails or prints something else fails uncoded. The token is not printed and its expiry is not looked at (`token inspect` does that). `storage.write.command` never runs: the only way to run it is to write a token, which replaces what the store holds and may prompt a keyring, so a broken write command still shows only at the next issue or refresh.
   - **`endpoints`**: first, the endpoints the grant needs must be configured, as `auth get` requires them: `endpoints.authorization` and `endpoints.token` for `authorization-code`, `endpoints.device-authorization` and `endpoints.token` for `device`, `endpoints.token` for both client credentials grants. The first missing one fails the row coded `endpoint-missing`, before any request. Then each configured endpoint among `endpoints.authorization`, `endpoints.device-authorization`, `endpoints.token` and `endpoints.revocation`, in that order, gets one request over a new connection opened exactly as the grants open theirs (io-oauth's `Oauth20ClientStd::connect`: the account's `tls` options and `alpn`, pimalaya-stream's 60-second read deadline). The request is the request line `HEAD <path and query> HTTP/1.1` with two headers, `Host: <host[:port]>` and `Connection: close`, and nothing else: no `Authorization` header, no client identifier, no cookie, no body. Ortie reads the status line of the answer and closes the connection. Any HTTP status line counts as answered, a 404 or a 405 included. The row fails on the first endpoint that does not answer (a name that does not resolve, a refused connection, a failed TLS handshake, no answer within the deadline, or an answer that is no HTTP status line), naming it: "Cannot reach endpoints.token at <url>: <why>". `endpoints.redirection` is sent nothing: it is Ortie's own loopback listener, or a page the browser lands on.
   A failed row ends the command with the partial-outcome status 3 after the report, as the family's checks do; the JSON report is the output, not a failure object.
3. **What the check proves**: the store can be read and holds a token; every configured endpoint's name resolves, its port accepts a connection, the TLS handshake passes with the account's TLS options, and an HTTP server answers there.
4. **What it does not prove**: that the provider accepts the client identifier, the secret, the key, the scopes or the redirection (only a grant proves that, and a grant spends the user's consent or a client credential, so a check never runs one); that the stored token is valid or unexpired; that the path of an endpoint is the right one (a 404 counts as answered); that a browser reaches `endpoints.authorization` (the check reaches it from where Ortie runs); and that `storage.write.command` works.
5. **Why HEAD, why no metadata.** A `HEAD` is a safe method (RFC 9110 section 9.2.1): no grant, no credential, no state on the server, and an answer without a body. A `POST` to the token endpoint would prove more of the OAuth answer but sends a grant attempt a server may log or count against the client. The authorization server metadata (RFC 8414) would need an issuer Ortie's configuration does not hold, and most providers serve it on another host than the token endpoint (Google), so deriving its URL would be guessing.
6. **Sessions.** In a REPL session `account check` runs over the session's account and runs the read command again rather than reading the token held in memory, which it leaves as it is. MCP lists no account tool.
7. **Schemas.** `ortie-account-list` and `ortie-account-check` join the registry, pimalaya-cli's types being the family's shared shapes like the failure object.

## Not done

- `account capability`: the family spec asks it of a binary with backends only.
- Checking the write command, the client or the token: see 4.

## Depends on

pimalaya-cli 0.5.1 (`pimalaya_cli::account`), required as 0.5.2.
