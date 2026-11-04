---
cairn: delta
change: repl-shell-split
---

## MODIFIED Requirements

### Requirement: Command grammar reuse
Each stdin line SHALL be split as a shell splits it, quotes and escapes included, then parsed with the same `token` and `auth` command grammar as the one-shot CLI (the account-less configuration wizard has no REPL form; `ortie configure` runs it). A line whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.
