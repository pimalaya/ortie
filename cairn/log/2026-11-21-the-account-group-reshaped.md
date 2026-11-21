---
cairn: log
change: the-account-group-reshaped
landed: 2026-11-21
---

# The account group reshaped for a binary without backends

The three account commands print Ortie's own types, no longer pimalaya-cli's `AccountListOutput`/`AccountCheckOutput`. `account list` prints `{"accounts": [{"name", "default", "grant"}]}` (src/account/list.rs). `account show` is new (src/account/show.rs): the account as the loader returns it, every `-c` layer merged and defaults filled, as TOML pasting back into a configuration or `{"account": {...}}` in camelCase, `client-secret.raw` printed `<redacted>` or `{"raw": null, "redacted": true}`; it loads the configuration again and runs no command, so a REPL line shows the account as it now loads. `account check` prints `{"account", "checks": [...]}` (src/account/check.rs): `storage`, then a new `token` row reading the stored token with no request (expired with no refresh token, on a grant other than the client credentials ones, fails coded the new `token-expired`; an empty store fails it `token-missing`; another storage failure fails it uncoded), then one row per endpoint the grant uses named by its configuration key, each checked on its own, `tls` on `https` rows only. The `HEAD` probe is unchanged. `GrantConfig`, `TlsProviderConfig` and `RustlsCryptoConfig` serialize as the configuration spells them; `ortie-account-show` joins the schema registry.

Two readings of the owner's design (2026-11-21) taken here: an expired token on a client credentials grant passes the token row, its grant running again on refresh; a configured endpoint the grant does not use is no longer sent a request, and a missing needed endpoint no longer stops the requests to the others.

Evidence: tests/account.rs rewritten to the new shapes plus the token-row, storage-failure and four `account show` tests (defaults filled, merged layers, no raw secret in text or JSON, the text pasted back loading the same JSON); 13 of its 15 tests and tests/repl.rs `a_line_takes_the_whole_grammar` failed on 020a0e8, the two passing ones being the unchanged TLS-string refusal and stdout-leak tests. src/account/show.rs unit tests load the text back through `Config::from_paths` into an equal account, the secret value included, and src/json_schema.rs checks the required keys of the three schemas. The `HEAD` request-head test keeps its exact assertions. Offline: loopback listeners and token files only.

Capabilities moved: account ("The list reads the configuration only" modified, "The show prints the account as loaded" added, "The check proves the storage, the token and the endpoints" replacing "The check proves the storage reads and the endpoints answer"), output ("Only Ortie's own types are described", "Commands carrying a schema", "A failure carries a stable code"), repl (the `account show` line).
