---
cairn: log
change: discovery-upgrades
date: 2026-11-03
---

# Discovery upgrades

`search_issuer` in src/wizard/search.rs falls back, for a URL whose RFC 8414 metadata does not resolve, to its RFC 9728 protected resource metadata: the grants of each authorization server it names are bound to the resource and narrowed to its scopes (`bind_resource`). `grants_of` offers only the flows a document's `grant_types_supported` lists, when it lists any. The wizard writes `pkce = "plain"` for an authorization code grant whose metadata lists `plain` alone (`suggested_pkce` in src/wizard.rs), S256 staying implicit otherwise. The manual fallback and the bare issuer retry of the original plan were dropped as obsolete: 2.1.0 made the wizard stop on nothing discovered and resolve every leftover issuer itself.

Capabilities moved: `discovery` (Input orients the flow and Issuers resolve to concrete grants modified, Metadata-driven PKCE added).

Verified: clippy clean, `cargo test --all-features` passed, among them `listed_grant_types_gate_the_offered_flows`, `a_resource_binds_the_grants_of_its_servers` and `only_a_plain_only_server_gets_a_pkce_line`. The wizard was not driven against a live server.
