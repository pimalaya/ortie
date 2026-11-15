---
cairn: change
id: config-conventions
status: landed
created: 2026-08-29
---

# Take the family's config conventions on TLS and defaults

## Why

A cross-product scan of the nine Pimalaya binaries measured where their configuration layers disagree, and Ortie is on the minority side of three of those splits.

Five products (himalaya, himalaya-tui, cardamum, neverest, sirup) expose `TlsConfig::into_tls(self, alpn)`, a single conversion taking the ALPN list as an argument. Ortie, calendula and carillon implement `From<TlsConfig> for Tls` instead, hardcoding an empty ALPN inside the conversion. The behaviour is the same today, but the shape is the problem: the ALPN decision is invisible at the call site, so a second call site negotiates none without anyone choosing that.

Ortie has no `alpn` key at all, so a user has no lever over it. That gap has a visible edge already: the wizard's discovery client offers `http/1.1` while the OAuth client offers nothing, against the very same hosts, and nothing in the configuration can reconcile the two.

The `default` field of the wizard fragment is skipped through `core::ops::Not::not` written fully qualified inline, against the repository's own always-`use` rule, where the rest of the family carries one `is_default` helper.

## What

`TlsConfig` becomes a public DTO the account config holds as-is, converted once by `into_tls(self, alpn)` in `Account::from`, which is the single place the config tree is flattened. The enum gains an `Auto` default variant naming the provider the binary was built with, which is what an omitted `tls` already selected, so the account field needs no `Option` and the conversion has no second spelling.

An `alpn` account option, `Vec<String>`, defaulting to empty. Empty is what Ortie offers today, so no account changes behaviour: an OAuth 2.0 endpoint is plain HTTPS and registers no identifier. The one meaningful override is `["http/1.1"]`, for a TLS middlebox refusing a handshake without ALPN. This mirrors himalaya's `sieve.alpn`, which is exposed with an empty default for the same reason: no registered identifier, but a user may still need one.

An `is_default<T: Default + PartialEq>` helper in `config.rs`, where the rest of the family keeps it, used by the wizard fragment's `default` field.

## Scope / non-goals

No shell-expansion work. Ortie already expands its two path fields at deserialize through its own `opt_shell_expanded_path`, which is the answer the scan settled on; it gains only a TODO naming the shared `pimalaya-config` helper it should become, since neverest and carillon hand-roll the same one.

No `Option<Vec<String>>` for `alpn`. The scan's sketch distinguished unset (protocol default) from an explicit empty list (ALPN disabled), which is meaningful for IMAP or JMAP where the protocol registers an identifier. OAuth over HTTPS registers none, so unset and empty are the same thing, and the flat `Vec<String>` himalaya and cardamum actually carry says it without a third state that decides nothing.

No `cert` key. The conversion keeps passing `None`, as it did before; pinning a certificate at an OAuth issuer is a separate question.
