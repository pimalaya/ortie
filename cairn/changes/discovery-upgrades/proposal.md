---
cairn: change
id: discovery-upgrades
status: landed
created: 2026-07-25
---

# Discovery upgrades

## Why
The wizard picked grants and the PKCE method by hand rather than from what the server advertises. Most of the metadata-driven discovery landed with 2.1.0 (issuers resolved through RFC 8414 into every grant whose endpoint is published, `auth discover` folded into bare `ortie`); what it left: a typed URL that is a protected resource rather than an issuer dead-ended, a server's `grant_types_supported` and `code_challenge_methods_supported` were ignored.

## What
A typed URL whose RFC 8414 metadata does not resolve SHALL be tried as a protected resource: its RFC 9728 metadata names the authorization servers, whose grants are bound to the resource and its scopes (io-pim-discovery's `rfc9728` feature, enabled by discovery-layering). The offered grants follow `grant_types_supported` when the document lists it. The wizard writes `pkce = "plain"` when the metadata lists `plain` alone, S256 staying the default otherwise, including for a document listing no method, since providers supporting PKCE omit it.

Dropped from the original plan: the manual entry pre-seeded with the issuer (the wizard configures only what it discovers since 2.1.0, stopping on the sample configuration) and the retry for unresolvable bare issuers (the wizard already re-resolves every issuer compose left bare, then drops it).
