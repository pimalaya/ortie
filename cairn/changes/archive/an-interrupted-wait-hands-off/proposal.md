---
cairn: change
id: an-interrupted-wait-hands-off
status: landed
created: 2026-11-19
---

# An interrupted wait hands off

## Why

Owner ruling of 2026-11-19, following the-flow-hardened: an interactive `auth get` no longer prints the state and the PKCE verifier, so a person who stops it with Ctrl-C while it waits for the redirection, the browser on another machine or the redirection lost, had no way left to finish the flow by hand and had to consent again.

## What

The wait installs pimalaya-cli's `signal::Shutdown` (the `signal` feature, ctrlc with its `termination` feature: SIGINT, SIGTERM and SIGHUP, Ctrl-C and Ctrl-Break on Windows) and runs the listener on a thread, polling the shutdown every 100 ms. On a first signal it prints the fallback's `auth resume` command, state, verifier and redirect URI included, and exits 130, pimalaya-cli's `signal::EXIT_SIGNALED`, the status the family gives a process a signal ended. A second signal ends the process with 130 at once, as the kit does.

Owner ruling of 2026-11-19, after the first landing: a REPL session installs no handler for the wait. Installed there, the handler outlived the wait and swallowed the next Ctrl-C at the prompt; the session keeps its earlier behaviour, a signal ending it with 130 and no handoff. Only the one-shot command line hands off.

### Breaking

None: 130 was already the shell's status for a process SIGINT killed.
