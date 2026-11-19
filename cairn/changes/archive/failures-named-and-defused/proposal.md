---
cairn: change
id: failures-named-and-defused
status: landed
created: 2026-11-19
---

# Failures named, server text defused

## Why

Two LOW findings of the CLI family analysis of 2026-11-15, re-checked open on 2026-11-16 and on 2026-11-19 at 58de3a9:

- ORT-8: a `--log-file` Ortie cannot open panicked at src/main.rs (`.expect("init logger")`), exit 101, where the other applications print the failure object and exit 1.
- ORT-9: a refresh `token show` makes on its own failed with a bare "connect <host>:<port>", no word that it was a refresh; src held two log statements; and the text output printed the token type, the scopes, the user code and the verification URIs as the server wrote them, escape sequences and bidi overrides included.

## What

- The logger's failure joins the command's result and goes through the failure report.
- The refresh's connection carries the context "Refresh access token error"; the refresh, the re-acquisition and the storage reads and writes are logged (`info!` for the actions, `debug!` for the storage).
- The text renderings defuse the server's strings with pimalaya-cli's `sanitize::text`.

### Breaking

None.
