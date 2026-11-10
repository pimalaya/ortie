---
cairn: change
id: text-output-ends-with-newline
status: landed
created: 2026-11-10
---

# Text output ends with a newline

## Why

The shared printer writes a text rendering as is, so each output type ends its own `Display` with a newline. `token inspect` did not, which left the shell prompt on its last line (the owner fixed it on 2026-11-10), and `token show` did not either, on purpose, for piping. The REPL papered over both by adding a newline after every text result, which doubled the blank line once `token inspect` carried its own. Checking the REPL in a terminal the same day also showed its `help` describing the line and the `auth` tree with developer notes.

## What

- `token show` ends the token with a newline, as `pass show` and `gh auth token` do; `$(...)` and pimalaya-config's secret commands (`trim_matches(['\r', '\n'])`) strip it.
- The REPL flushes a result without adding a newline.
- The REPL's line parser and `auth` tree carry user-facing help.
