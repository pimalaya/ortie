---
cairn: delta
change: discovery-upgrades
---

## ADDED Requirements

### Requirement: Metadata-driven PKCE
The wizard SHALL write `pkce = "plain"` for an authorization code grant whose server metadata lists `plain` as its only code challenge method, and no `pkce` line otherwise, S256 being the default.

## MODIFIED Requirements

### Requirement: Input orients the flow
A single prompt SHALL accept an email address, a bare domain, or an issuer URL, and nothing else. An email or bare domain runs io-pim-discovery's parallel discovery; a URL resolves its RFC 8414 authorization server metadata into the grants it advertises, or, when it has none, its RFC 9728 protected resource metadata into the grants of the authorization servers it names, each bound to the resource and narrowed to its scopes. The wizard SHALL NOT offer any hand-entry of OAuth 2.0 endpoints.

### Requirement: Issuers resolve to concrete grants
A discovered `OauthIssuer` entry, and a typed issuer URL, SHALL be resolved through the issuer's RFC 8414 metadata into every grant it advertises: the authorization code grant when an authorization endpoint is published, the device authorization grant when a device authorization endpoint is, and both when both are, since RFC 8414 section 2 and RFC 8628 section 4 let a server advertise them side by side and the choice between them belongs to the pick list. A document listing `grant_types_supported` SHALL offer only the flows it names. Both need the token endpoint, so a document without one advertises nothing. An issuer whose metadata cannot be resolved into any grant SHALL be dropped from the pick list rather than emitted as a bare issuer comment.

## REMOVED Requirements
