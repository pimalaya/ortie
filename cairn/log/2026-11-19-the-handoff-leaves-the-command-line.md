---
cairn: log
change: the-handoff-leaves-the-command-line
landed: 2026-11-19
---

# The handoff can stay off the command line

The rest of finding ORT-4 of the CLI family analysis of 2026-11-15, as the owner ruled twice on 2026-11-19: the handoff off the command line, then read only when asked, never implicitly from stdin.

`auth listen` and `auth resume` take `--handoff <PATH>`, the object `auth get --json` prints, from a file or, for `-`, from stdin, the family's spelling for a JSON input on stdin (himalaya `jmap request`, cardamum's JSON inputs). src/auth/handoff.rs `AuthHandoff` reads at most 64 KiB, refuses an object that does not parse by its line and column, never a value, and fills `--state`, `--pkce` and an omitted `--redirect-uri`. The flag conflicts with `--state` and `--pkce` in clap (`usage-invalid`, exit 2; no family precedent has a flag override another here). src/auth.rs refuses it on the device and client credentials grants, and `--handoff -` in a REPL session (`Session::stdin_handoff`), whose stdin is its lines. Without the flag neither command reads stdin. The help of both commands says `--state` and `--pkce` are visible to other local users.

Evidence, tests/auth_listen.rs, each failing on the tree before this change (the first two on the implicit read, which `--handoff` replaced): `a_handoff_on_stdin_is_listened_for_and_stored` (`--handoff -`, no flow value on the listener's command line), `a_handoff_on_stdin_is_resumed` (a forged redirect refused `state-mismatch` by the state of `--handoff -`, then the exchange through `--handoff <file>`), `without_the_handoff_flag_an_open_stdin_is_left_alone` (an open stdin, the command done within 5 s; the implicit read waited on it), `the_handoff_flag_refuses_the_state_and_pkce_flags`. `a_repl_resume_takes_no_handoff_from_its_lines` guards the REPL.

Parked: Ortie writing a 0600 handoff file, proposal a-private-handoff-file, until the kit's private-create helper (KIT-4).

Capabilities moved: auth ("The handoff can stay off the command line", added).
