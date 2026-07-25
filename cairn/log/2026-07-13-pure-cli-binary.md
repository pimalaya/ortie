---
cairn: log
change: pure-cli-binary
landed: 2026-07-13
---

# M0: pure CLI binary

Removed `src/lib.rs` and the `cli`/`client` cargo features; the binary always builds and declares `mod cli`. Remaining features are the TLS providers (rustls-ring default), `vendored`, and `notify`; feature-gated deps became plain deps and the bin target lost required-features. `Cargo.toml` description became "CLI to manage OAuth 2.0 tokens", dropping the docs.rs metadata, documentation field, api-bindings category and io-free/coroutine keywords. The README dropped its docs.rs badge and states Ortie exposes no library API, redirecting to io-oauth. The lib removal also let five never-called pub helpers and the never-read `Account.default` field go, and the no_std-style `alloc::`/`core::` imports became `std`.

Spec: `packaging` (pure CLI binary, TLS-only feature set).
