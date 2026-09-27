---
cairn: log
change: config-conventions
landed: 2026-08-29
---

# Took the family's TLS conversion, and gained an ALPN key

A cross-product scan measured where the nine Pimalaya binaries disagree on their configuration layers, and put Ortie on the minority side of three splits. All three landed together, none of them changing what an existing account does.

`impl From<TlsConfig> for Tls` is gone, replaced by `TlsConfig::into_tls(self, alpn)`, which is what five of the nine already expose. The old conversion hardcoded an empty ALPN in three of its arms, so the decision was invisible from where it mattered: a second call site would have negotiated none without anyone choosing that. The account config now holds the `TlsConfig` DTO rather than an already-built handle, and `Account::from`, the single place the config tree is flattened, is the single place the conversion happens.

The enum gained an `Auto` variant, its default, naming the provider the binary was built with. That is what an omitted `tls` already selected; giving it a name is what lets the account field stay non-optional, and the conversion have exactly one spelling rather than a match on `Option` with a hand-built handle in the `None` arm. `tls = "auto"` therefore parses now and did not before, which is the one thing here that widens what the file accepts.

`alpn` is new. It defaults to empty, which is what Ortie has always offered on its OAuth connections, so no account changes behaviour. Whether to expose it at all was the open question in the scan, since an OAuth 2.0 endpoint is plain HTTPS and registers no ALPN identifier the way IMAP or ManageSieve do. Two things decided it. himalaya exposes `sieve.alpn` with an empty default for exactly that reason, no registered identifier but a user who may still need one. And Ortie itself already offers `http/1.1` from the wizard's discovery client while offering nothing from the OAuth client, against the same hosts, with nothing in the configuration able to reconcile the two. `["http/1.1"]` is the one meaningful override, for a TLS middlebox refusing a handshake without ALPN.

The scan sketched the key as `Option<Vec<String>>`, distinguishing unset from an explicit empty list. That distinction earns its keep where the protocol has a default identifier to fall back to; here unset and empty are the same thing, so the flat `Vec<String>` that himalaya and cardamum actually carry is what landed.

`is_default` joined `config.rs`, where the rest of the family keeps it, and replaced `core::ops::Not::not` written fully qualified inline on the wizard fragment's `default` field, against the repository's own always-`use` rule. The emitted fragment is unchanged: it already omitted a false default, just through a predicate nobody else uses.

Not done: shell expansion, which Ortie already gets right. Its two path fields expand at deserialize through a private `opt_shell_expanded_path`, which is the answer the scan settled on for the whole family. It gained a TODO naming the shared `pimalaya-config` helper it should become, since neverest and carillon hand-roll the same one.

Verified: build, fmt and clippy clean; 38 unit tests and 6 integration tests pass, two of them new, covering an account naming both keys and an account naming neither. The live four-account configuration still loads and resolves every account.

Spec updated: config (ADDED: One TLS conversion taking the ALPN list, ALPN is a config key, A defaulted field is absent from the generated account).
