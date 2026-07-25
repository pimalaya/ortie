---
cairn: log
change: persistent-repl
landed: 2026-07-25
---

# Persistent REPL session

Added `ortie repl`: a persistent session bound to one account that resolves the stored token once and answers `token` commands over stdin/stdout, so a keyring that confirms disclosure per process is unlocked a single time instead of on every command. The persistence is private to the spawning process (the stdio pipe is the authorization boundary), so unlike a shared socket agent it does not hand the token to any other process.

`Account` now holds the resolved token in an `Option`, built `None` and memoized by `resolve_token` on first use; the `token` dispatch moved to `&mut Account` so the same account (and its cached token) carries across loop iterations, while the auth grants keep building `None`. On refresh the new token is persisted to storage before the in-memory copy is replaced. Because io-oauth is `no_std` and cannot read a clock, `write_to_storage` stamps `issued_at` with the local wall clock at receipt (overriding the server `Date` header) so expiry stays on one clock, and a missing `expires_in` now defaults to one hour instead of never expiring. Results are flushed per command so a piped consumer reads one token per line on stdout with errors on stderr; the `\nortie> ` prompt shows only on a TTY.

Spec: `repl` (new: persistent session, command grammar reuse, interactive versus piped output), `token` (MODIFIED expiry with skew for the one-hour default; ADDED local issuance timestamp).
