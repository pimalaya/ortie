---
cairn: log
change: an-interrupted-wait-hands-off
landed: 2026-11-19
---

# An interrupted wait hands off

Owner ruling of 2026-11-19. src/auth/get.rs runs the interactive wait for the redirection on a thread and polls pimalaya-cli's `signal::Shutdown` every 100 ms; on a first SIGINT, SIGTERM or SIGHUP it prints the fallback's `auth resume --state=… --pkce=… --redirect-uri=…` command and exits 130 (`signal::EXIT_SIGNALED`, the family's status for a process a signal ended; no other convention exists in the kit). Cargo.toml turns on pimalaya-cli's `signal` feature, which brings ctrlc 3.5 with nix 0.32 and, on macOS only, objc2 and dispatch2 into the lock; `cargo deny check` passes bans and licenses.

Evidence: tests/auth_listen.rs `an_interrupted_interactive_wait_prints_the_resume` (Linux only: the binary runs in a terminal util-linux's `script` allocates, gets SIGINT while waiting, and must print the resume line and exit 130) failed on the tree before this change, which died on the signal with no handoff.

In a REPL session no handler is installed (owner ruling of 2026-11-19, `Session::interrupt_handoff`): the shutdown, once installed, lasts the process, and in a session it outlived the wait and swallowed the next Ctrl-C at the prompt. There the wait runs on the main thread as before and a signal ends the session with 130, no handoff. Guard: tests/auth_listen.rs `an_interrupted_repl_wait_ends_the_session` (a REPL in a `script` terminal, `auth get`, one SIGINT: exit 130, no resume line), failing on the first landing, which printed the handoff in the session.

Capabilities moved: auth ("An interrupted wait hands off", added).
