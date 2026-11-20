---
cairn: change
id: the-contract-audit-rulings
status: landed
created: 2026-11-20
---

# The contract audit rulings

## Why

The CLI contract audit of 2026-11-20 found Ortie conforming but for a few points, and the owner ruled on the family-wide ones the same day: no by-name refusal of a removed configuration key anywhere (A5, which supersedes ruling D4's by-name `tls` refusal), one TLS table spelled as himalaya spells it and named on the check row (A1), and `account check` printing a list of accounts in every binary (A2). pimalaya-cli carries the kit side (the check row's `tls`, the list of accounts, the bare invocation under `--json`).

## What

- ORT-X1, A5: the `hooks` table and the `tls` string are refused by serde alone, as an unknown field and a value of the wrong type; `HooksRemoved` and the custom `tls` visitor go.
- A1: `tls.client.cert` and `tls.client.key`, the client certificate pimalaya-stream presents under both providers; `account check` prints the kit's list of accounts, the `endpoints` row carrying the TLS configured once one checked endpoint is `https`, the `storage` row `null`.
- ORT-X2: a `--handoff` on a grant without one and an `auth listen` with no redirection to bind are refused through `pimalaya_cli::error::usage_error`, exit 2.
- ORT-X3: `--handoff -` inside a session goes through `pimalaya_cli::session::check_stdin`, coded `stdin-taken`, new in the vocabulary; the session's own `stdin_handoff` flag goes, the kit knowing when stdin is claimed.
- ORT-X6: `token get` and `auth continue` become hidden aliases.
- ORT-X8: MIGRATION.md lists every new schema key, the CHANGELOG one pimalaya-cli line.
- Kit item B: a bare `ortie` answers through `pimalaya_cli::clap::meet_bare_invocation`, so `ortie --json` alone is refused coded `usage-invalid`, exit 2.
- Kept as they are: ORT-X5 (parked under a-private-handoff-file), ORT-X7 (130 on an interrupted interactive `auth get`), ORT-X9 (COH-1, no kit change), ORT-X11 (an-account-proxy).
