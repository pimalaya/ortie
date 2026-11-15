---
cairn: log
change: account-group
landed: 2026-11-15
---

# The `account` group: `account list` and `account check`

`ortie account list` (alias `ls`) prints pimalaya-cli's `AccountListOutput` from the configuration alone, `backends` and `notBuilt` empty. `ortie account check` prints `AccountCheckOutput<ErrorCode>` with two rows: `storage` runs `storage.read.command` and parses the token (empty store coded `token-missing`), never the write command; `endpoints` fails coded `endpoint-missing` on an endpoint the grant needs that the account leaves out, then sends each configured endpoint but the redirection one `HEAD <path and query> HTTP/1.1` carrying only `Host` and `Connection: close`, over io-oauth's `Oauth20ClientStd::connect`, any HTTP status line counting as answered. A failed row exits 3 after the report. The runtime `Account` gains `name`; `ortie-account-list` and `ortie-account-check` join the schema registry.

Evidence: tests/account.rs runs the binary against a local listener and a token file (the exact request heads and their order, the write command never run, both rows on a failure with exit 3, `endpoint-missing` before any request, a non-HTTP answer, `account-missing`); every test fails on the previous tree, which had no `account` command. No provider was reached.

Capabilities moved: account (new: "The list reads the configuration only", "The check proves the storage reads and the endpoints answer"), output ("Only Ortie's own types are described", "Commands carrying a schema").
