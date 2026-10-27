---
cairn: log
change: resume-hint-account
landed: 2026-10-27
---

# The printed resume command keeps its account

`auth get` printed `ortie auth resume …` without the `-a` and `-c` it ran with, so pasting it resumed the default account (#16): a Google code sent to a Microsoft token endpoint, or a token stored under the wrong account when both share a client. `resume_invocation` (src/auth/get.rs) now builds the printed prefix from the global flags, single quoted, for the authorization code and device hand-offs, the REPL passing its own.

Capabilities moved: auth (Manual resume fallback).

Verified: `cargo clippy --all-targets` and `test`, a build with `rustls-ring` alone, and `auth get` on a two-account scratch configuration printing `ortie -c '…' -a 'work' auth resume …`.
