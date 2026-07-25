---
cairn: delta
change: discovery-upgrades
---

## ADDED Requirements

### Requirement: Metadata-driven issuer resolution
On `auth discover <URI>`, the wizard SHALL resolve RFC 8414 issuer metadata first, fall back to RFC 9728 resource metadata then its authorization servers, and only then to manual entry pre-seeded with the issuer.

### Requirement: Metadata-driven grant and PKCE choice
The offered grants SHALL be driven by `grant_types_supported` and `device_authorization_endpoint`, and the suggested `pkce` value by `code_challenge_methods_supported`.
