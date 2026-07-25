---
cairn: log
change: auth-in-repl
landed: 2026-07-25
---

# Auth commands in the REPL

Brought `auth get` and `auth resume` into the REPL so a whole session can authorize, use, and refresh a token behind one keyring unlock. `AuthGetCommand::execute`, `AuthResumeCommand::execute` and their internal `execute_device` moved from consuming `Account` by value to `&mut Account`, so the REPL's single account carries across iterations (the one-shot `AuthCommand::execute` now resolves `let mut account` and lends it). A `ReplAuthCommand` (get/resume) was added alongside the token branch, dispatched against the session account; the account-less `discover` wizard has no REPL form. Because auth runs against the same account, `write_to_storage` updates the in-memory token, so a token issued by `auth get`/`auth resume` is served by a following `token show` with no re-read — covered by a new integration test driving `auth get` → `auth resume` → `token show` through the `repl` binary against a mock AS. The existing `stdout().is_terminal()` branches mean interactive sessions open the browser / poll and piped sessions print the manual handoff, with no REPL-specific TTY code.

Spec: `repl` (MODIFIED command grammar reuse to include `auth`; ADDED auth runs against the session account).
