---
cairn: change
id: discovery-upgrades
status: draft
created: 2026-07-25
---

# Discovery upgrades

## Why
`auth discover <URI>` currently dead-ends on `://` inputs (straight to manual entry) and the grant/PKCE choices are hand-picked rather than driven by what the server advertises. io-pim-discovery can already resolve RFC 8414 issuer metadata and enable RFC 9728, so metadata-driven discovery is wiring, not new protocol work.

## What
Enable io-pim-discovery's rfc9728 feature. On `auth discover <URI>`, resolve RFC 8414 issuer metadata first (`ComposeClientStd::oauth_server`), fall back to RFC 9728 resource metadata then its authorization servers, and only then to manual entry pre-seeded with the issuer. Surface grant choice from metadata: `grant_types_supported` and `device_authorization_endpoint` drive which grants are offered, and `code_challenge_methods_supported` drives the suggested `pkce` value. Unresolvable bare issuer picks get a metadata-resolution retry instead of dead-ending.
