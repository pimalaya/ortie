---
cairn: change
id: auth-in-repl
status: landed
created: 2026-07-25
---

# Auth commands in the REPL

## Why
The REPL already resolves the account and runs `token` commands over stdin/stdout, on a TTY or piped. Extending it to `auth get`/`auth resume` lets a user do the whole lifecycle in one session — authorize, then use and refresh the token — behind a single keyring unlock, and lets an application drive the grant without reimplementing it.

## What
Convert `AuthGetCommand::execute` and `AuthResumeCommand::execute` (and their internal `execute_device`) from consuming `Account` by value to `&mut Account`, so the REPL's single account carries across iterations. Add a `token`-sibling `auth` branch to the REPL grammar exposing `get` and `resume` (not the account-less `discover` wizard, which bare `ortie` runs). Because auth runs against the same account, `write_to_storage` updates the in-memory token, so a token issued by `auth get`/`auth resume` is served by a following `token show` with no re-read. The commands already branch on `stdout().is_terminal()`, so interactive REPL sessions open the browser / spawn the redirect listener (or poll the device grant) and piped sessions print the manual handoff — no REPL-specific TTY handling.

## Rejected
- Reusing `AuthCommand::execute` in the REPL: it re-resolves the account per leaf, which would desync the REPL's cached token from an auth-issued one. A small `ReplAuthCommand` dispatching against the shared `&mut Account` keeps them coherent.
