---
cairn: log
change: discovery-layering
date: 2026-11-03
---

# Typed resource and scopes from discovery

An account now carries `resources` (RFC 8707), sent by `auth get` on the authorization URL and by `auth resume` on the code exchange through io-oauth's new typed `resource` fields. The wizard reads the chosen grant's `resource` and `scopes_supported`, both filled by io-pim-discovery from the RFC 9728 metadata of the service endpoint: it writes the resource, and preselects the resource's scopes when the grant requests none. Merging per-service grants keeps the first resource in compose priority order, the JMAP session for Fastmail, since several indicators on one request are unverified there. `fill_provider_defaults` and the Fastmail scope list in src/wizard.rs are gone, and the wizard no longer writes `extras`. io-pim-discovery gains its `rfc9728` feature here. Both libraries are path-patched until their releases.

The relocation of the RFC 8414/9728 types into io-oauth moved to io-pim-discovery's draft change oauth-metadata-relocation.

Capabilities moved: `config` (Resource indicators added, Extras passthrough modified), `discovery` (Resource-bound grants added), `provider-quirks` (Fastmail resource and redirect modified).

Verified: `cargo clippy --all-features --all-targets` and the `rustls-ring`-only build clean, `cargo test --all-features` passed, among them `merged_grant_takes_the_first_resource_with_its_scopes` and the resource case of `a_discovered_grant_becomes_its_config_shape`. Fastmail's resource metadata was checked live through io-pim-discovery; the full wizard was not driven.
