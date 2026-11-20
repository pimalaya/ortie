---
cairn: log
change: the-contract-audit-rulings
landed: 2026-11-20
---

# The contract audit rulings

Ortie took its share of the CLI contract audit of 2026-11-20 and of the owner's rulings on it, against pimalaya-cli's unreleased working tree (a path patch until the kit's next release).

## What reaches users

- A `hooks` table reads "unknown field `hooks`" and a `tls` string "invalid type: string", where both named MIGRATION.md (ruling A5, superseding D4's by-name `tls` refusal).
- `tls.client.cert` and `tls.client.key` present a client certificate to the authorization server (ruling A1); pimalaya-stream 0.4 already carries it to rustls and native-tls through io-oauth's connect.
- `account check --json` prints `{"accounts": [{"account", "backends"}]}`, each row with `tls`, filled on `endpoints` once one checked endpoint is `https` (`{provider, crypto, cert, clientCert}`), `null` on `storage`; the text appends `, TLS <parts>` to the endpoints row.
- `auth listen` with no redirection and `--handoff` on a device or client credentials account exit 2 coded `usage-invalid` (ORT-X2), where they exited 1 uncoded.
- `--handoff -` inside a REPL session reads "Cannot read input from stdin inside a session, pass it as an argument or a file path", coded `stdin-taken` (ORT-X3).
- `ortie --json` alone exits 2 coded `usage-invalid`, where it printed the help on stdout.
- `token get` and `auth continue` leave the help, still accepted (ORT-X6).

## Capabilities moved

config, account, auth and output (see the change's delta).

## Verified

`cargo test` (every new test failed on the code before the change, run from a snapshot of HEAD with the new tests copied in), `cargo clippy --all-targets -D warnings` under the default features, `native-tls` and `rustls-aws` alone, `cargo fmt`. The regenerated `json-schema` differs from the audit's dump in `ortie-account-check` (the list of accounts and `tls`) and `ortie-error` (`stdin-taken`) alone.
