---
cairn: delta
change: auth-in-repl
---

## MODIFIED Requirements

### Requirement: Command grammar reuse
Each stdin line SHALL be parsed with the same `token` and `auth` command grammar as the one-shot CLI (the account-less `auth discover` wizard has no REPL form; bare `ortie` runs it). A parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

## ADDED Requirements

### Requirement: Auth runs against the session account
`auth get` and `auth resume` in the REPL SHALL run against the session's in-memory account, so a token they issue updates the cached token and is served by a following `token show` without re-reading storage.
