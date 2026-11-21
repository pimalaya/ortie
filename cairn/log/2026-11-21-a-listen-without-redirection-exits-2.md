---
cairn: log
change: a-listen-without-redirection-exits-2
landed: 2026-11-21
---

# A listen without redirection exits 2

Owner ruling of 2026-11-21, revising the rulings before 3.0.0: an `auth listen` given no redirection by `--redirect-uri`, a handoff `redirectUri` or `endpoints.redirection` exits 2 coded `usage-invalid`, no longer 1 coded `endpoint-missing`. With no configured redirection, `auth get` binds an ephemeral port, so only the redirect URI it printed can finish the flow: the fix is on the command line. An `endpoints.redirection` that is not an http URI on a loopback host keeps exiting 1 coded `redirection-unsupported`.

## What landed

- src/auth/listen.rs: the refusal is `usage-invalid`, "Missing --redirect-uri, pass the one auth get printed"; the help states it.
- tests/auth_listen.rs: `a_redirection_the_config_lacks_or_cannot_listen_on_is_refused` asserts exit 2 coded `usage-invalid` for the missing case, exit 1 coded `redirection-unsupported` for the other two.

## Spec

auth ("Loopback listen"), output (the `endpoint-missing` row no longer names `auth listen`).

## Verified

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (140 passed).
