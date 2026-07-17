# Ortie docs

Living design documents and plans for Ortie. The repository architecture itself is documented in the src/main.rs header, the same way lib.rs documents io-oauth.

[oauth21-plan.md](./oauth21-plan.md) tracks the OAuth 2.1 readiness and multi-grant work: the settled design decisions (grant selection, pkce shape, extras passthrough, wizard write-back), the milestone sequence (device grant, discovery upgrades, revocation) and the landed history.

[discovery-layering.md](./discovery-layering.md) is a cross-crate design note (io-oauth and io-pim-discovery, surfaced through Ortie): the spec-provenance rule for where discovery code lives, the RFC 8707 resource trap that both Ortie and cardamum-android hit, and the domain-scoping refactor that would fix it for good. Until that lands, it documents the config stopgap that makes Fastmail work today.

[providers.md](./providers.md) collects the per-provider OAuth 2.0 quirks (Google, Microsoft, Fastmail) learned while testing the wizard: the endpoints and scopes that actually work, the "This app is blocked" / `invalid_scope` traps, and each provider's redirect and desktop-completion caveats.
