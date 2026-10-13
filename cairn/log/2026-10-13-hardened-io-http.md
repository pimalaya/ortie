---
cairn: log
change: hardened-io-http
landed: 2026-10-13
---

# io-http pinned to its hardening fix

`[patch.crates-io]` pins io-http, reached through io-oauth and io-pim-discovery, to e181530, unreleased, until a 0.5 release carries it. A token, revocation or discovery response with a head over 64 KiB or a body over 256 MiB now fails instead of being buffered, a connection closed mid-body fails with `Eof` instead of spinning on `WantsRead`, and a chunk size near `usize::MAX` no longer panics. Ortie's own code is unchanged.

Capabilities moved: none.

Verified: `cargo build`, `clippy --all-targets` and `test`, with `rustls-ring` alone and with the defaults (wizard included).
