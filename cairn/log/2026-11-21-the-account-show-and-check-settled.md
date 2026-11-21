---
cairn: log
change: the-account-show-and-check-settled
landed: 2026-11-21
---

# The account show and check settled for 3.0.0

Owner decisions of 2026-11-21, landed on aaab579. `account show` text (src/account/show.rs) is a plain view: `Account: <name>` then one pimalaya-cli `fields::Fields` field per property labelled by its configuration key, `alpn`, `scopes` and `resources` one value per line, commands as pimalaya-config's `CommandConfig` displays them, every label and value through `sanitize::text`, `client-secret.raw: <redacted>`; the TOML text and its paste-back promise are gone. Its JSON is unchanged but for `tls`, left out when the account sets no TLS key (it printed `{"rustls": {}}`). `account check` (src/account/check.rs) prints one `{"event": "check-finished", "account", "check", "ok", "code", "error", "tls"}` event per row as soon as it is checked, then its unchanged report, through the kit's `Printer::event`: one flushed JSON line each, a `<check>: OK` or `<check>: FAIL (<why>)` text line on a terminal only. Exit codes are unchanged. src/json_schema.rs registers `AccountCheckEvent` under `ortie-account-check` in the events map.

Mirrored: the event enum shape of neverest's `SyncEvent` (src/sync/event.rs, `tag = "event"`, kebab-case names, camelCase fields, a `*-finished` event carrying `code` and `error`) and carillon's `WatchStreamEvent` (src/stream.rs, an `account` on each event, a one-line sanitized text), the registry's `insert_events` as both register theirs, and the result listing every row again as neverest's report does. Not mirrored: neverest `sync` and carillon `watch` stream text events to a pipe (`StdoutPrinter::streaming`); pimalaya-cli cairn/spec/output.md, "A long-lived command streams events", forbids it for a check ("a command ending on its own ... SHALL NOT"), so `account check` text events reach a terminal only, as neverest `check` prints its progress lines. The events carry the row's keys only, without the `sources` list neverest's `collection-finished` and the kit's recovered-error events carry, since the row's `error` already holds the whole chain.

Evidence: on aaab579, 11 of 15 tests of tests/account.rs (each check test now asserts one event per row before the report through a `checked` helper, and the show tests the plain view), tests/repl.rs `a_line_takes_the_whole_grammar`, two src/account/show.rs unit tests and two src/json_schema.rs tests failed; all pass now, with a new unit test of the event text. The `HEAD` request-head test keeps its exact assertions. Offline: loopback listeners and token files only. A pty run (`script`) showed the text events before the report; a piped one the report alone.

Capabilities moved: account ("The show prints the account as loaded", "The check proves the storage, the token and the endpoints"), output ("Commands carrying a schema", "Only Ortie's own types are described"), repl ("Command grammar reuse").
