---
cairn: change
id: the-account-show-and-check-settled
status: landed
created: 2026-11-21
---

# The account show and check settled for 3.0.0

## Why

The account group reshaped earlier on 2026-11-21 (cairn/log/2026-11-21-the-account-group-reshaped.md) left two shapes the owner decided against before 3.0.0, the same day. `account show` printed its text as TOML pasting back into a configuration, where every other detail view of Ortie and the family prints aligned `Label: value` fields (pimalaya-cli `fields::Fields`, as `token inspect` and `account check` do), and its JSON printed a meaningless `"tls": {"rustls": {}}` default on an account setting no TLS key. `account check` runs a storage command and several network probes, a long process, yet printed nothing until every check ended.

## What

1. `account show` text: a plain view built with the kit's text utilities, `Account: <name>` then one `fields::Fields` field per property, labelled by its configuration key (`client-id`, `endpoints.token`, `storage.read.command`), a list one value per line as `token inspect` prints scopes, a command as pimalaya-config's `CommandConfig` displays it, every label and value through `sanitize::text`. A raw secret prints `<redacted>`. The round-trip promise and its tests go.
2. `account show` JSON: unchanged, except `tls` is left out when the account sets no TLS key.
3. `account check`: the family's events-then-result pattern (pimalaya-cli cairn/spec/output.md, "Events precede the result"), as neverest `sync` and carillon `watch` print it: one `{"event": "check-finished", "account", "check", "ok", "code", "error", "tls"}` event per row as soon as it is checked, then the report, unchanged. Text events print on a terminal only, a check not being a streaming command by the kit's rule; exit codes unchanged. The events schema registers as `ortie-account-check.events`.
4. REPL: a line prints the events then the report, as the kit's REPL printer does for any command; no MCP tool is added.
