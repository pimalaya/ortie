---
cairn: change
id: repl-line-json
status: landed
created: 2026-11-04
---

# A REPL line takes `--json`

## Why

`--json` is a global flag of the one-shot CLI, so `token inspect --json` works outside the REPL, but the REPL's line grammar did not carry it: inside a session the same line failed with "unexpected argument '--json'". The REPL claims the one-shot grammar, so a line that works in a shell should work in a session.

## What

A line takes `--json`, which switches that line's result, or its failure, to JSON; the next lines keep the session's output. A `--json` session stays JSON throughout, there being no flag to switch a line back to text. `-a` and `-c` stay refused: the account and the configuration are the session's.
