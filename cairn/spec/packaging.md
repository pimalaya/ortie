---
cairn: spec
capability: packaging
status: current
---

# Packaging

Ortie is a thin, config-driven front-end over io-oauth (the OAuth engine) and io-pim-discovery (PIM service discovery). It ships as a binary only, with no library surface, and adopts OAuth 2.1-compatible defaults without modelling a protocol version.

### Requirement: Pure CLI binary
Ortie SHALL build as a binary with no public library API. The lib target and the `cli`/`client` cargo features do not exist; the README and CHANGELOG redirect the rare library user to io-oauth.

### Requirement: TLS-only feature set
The cargo features SHALL be the TLS providers (rustls-ring by default), `vendored`, and `notify`. The non-TLS features (`oauth2`/`rfc6749`, `command`, `cli`, `client`) are dropped.

### Requirement: No version switch
Ortie SHALL NOT expose a `version` config option for OAuth 2.0 versus 2.1. 2.1 is a constraint profile of 2.0 with identical wire messages; Ortie adopts 2.1-compatible defaults (S256 PKCE, exact redirect matching, refresh rotation) with explicit 2.0 escape hatches (`pkce = false`, `pkce = "plain"`). A hard-strict mode, if ever needed, would land later as a single `strict = true` flag.

### Requirement: Licence
Ortie SHALL be dual-licensed MIT OR Apache-2.0.
