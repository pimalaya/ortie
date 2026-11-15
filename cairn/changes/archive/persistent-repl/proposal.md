---
cairn: change
id: persistent-repl
status: landed
created: 2026-07-25
---

# Persistent REPL session

## Why
In one-shot mode every `token show` is a fresh process, so it re-reads the secret store from scratch. On a keyring that confirms disclosure per process (kwallet, a hardened secret-service), the user gets a fresh authorization prompt at random times all day. A persistent session resolves the secret **once** and reuses it in memory, collapsing N prompts into one (plus one per refresh write, which is unavoidable because a rotated refresh token must be persisted).

## What
Add `ortie repl`: a persistent session bound to one account (`--account` or default, like any other command). It reads a command per line on stdin, runs it against an in-memory account, and writes the result on stdout, looping until EOF. It reuses the existing `token` command grammar (parse each line, never exit the loop on a bad line). When stdin is a TTY it renders a prompt; when piped it uses a plain line protocol (results on stdout, errors and prompts on stderr) so applications can drive it.

## Design

**REPL, not a socket daemon.** The request is for both CLI flexibility *and* per-process authorization. Those are mutually exclusive for a shared daemon: a socket agent (gpg-agent shape) hands the token to any process of the same uid (ambient access). A REPL keeps the persistent secret-holder **private to the process that spawned it** — the stdio pipe is the authorization boundary — so it recovers per-process scope. One repl per account, owned by its parent; on exit the pipe closes and the in-memory secret is zeroized.

**Account holds the resolved token.** `Account` gains `token: Option<...>`, built `None` (the `Option` states the intent plainly: a token may or may not be resolved yet). A command that needs the token resolves the read storage command on the fly and feeds the account; the resolved token is memoized, so the read command runs once. The auth grants keep building `None` (they issue a token, they do not read one), so this does not touch `auth get`/`auth resume`. The token dispatch moves from consuming `Account` by value to `&mut Account`, so the same account (and its cached token) carries across loop iterations.

**Refresh writes the keychain first, then memory.** On refresh, the new token is persisted to storage **before** the in-memory token is replaced, so a rotated refresh token can never be lost and a crash cannot strand the account with an in-memory-only rotation the store never saw.

**Local issuance timestamp.** io-oauth is `no_std` and cannot read a clock, so it can only lift `issued_at` from the server `Date` header. Ortie stamps `issued_at` with its own wall clock at receipt, on the value it persists and caches, overriding the header. This keeps both ends of the expiry comparison (`issued_at` and the later `now`) on one clock, so server/local skew cancels and the only error is network latency (the safe direction). Without it, a long-lived repl could serve a stale token forever when a provider omits a usable `Date`.

**Missing `expires_in` defaults to one hour.** When the server sends no lifetime, expiry is otherwise unknowable and a repl would never refresh. A solid 3600s default makes the session refresh roughly hourly instead of never.

## Rejected
- A socket/daemon agent: gives ambient uid-scoped access, the exact model the request wants to avoid.
- A separate cached-token field beside the read command: two representations of one thing; the `token: Option` on `Account` is the single resolved-data home.
- Multi-account in one repl: a repl is bound to one account like every command; run one per account.
