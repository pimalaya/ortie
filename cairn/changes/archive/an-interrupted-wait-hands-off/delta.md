---
cairn: delta
change: an-interrupted-wait-hands-off
---

## ADDED Requirements
### Requirement: An interrupted wait hands off
When a termination signal (SIGINT, SIGTERM, SIGHUP; Ctrl-C or Ctrl-Break on Windows) reaches an interactive `auth get` waiting for the redirection, it SHALL print the `auth resume` command a fallback prints, with the state, the PKCE verifier and the redirect URI, then exit 130, pimalaya-cli's status for a process a signal ended (`signal::EXIT_SIGNALED`). In a REPL session no handler SHALL be installed: the signal ends the session with 130 as it would without Ortie, no handoff printed, and a later Ctrl-C at the prompt is not swallowed.
