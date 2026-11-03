---
cairn: change
id: discovery-layering
status: landed
created: 2026-07-25
---

# Discovery layering: typed resource and scopes

## Why
Fastmail refused to work in Ortie for the same reason it once refused in cardamum-android: both consumers had to re-derive the same provider knowledge by hand. Fastmail's authorize endpoint hard-requires an RFC 8707 `resource` parameter; io-oauth modelled the *failure* (`InvalidTarget`) but no request type carried a `resource` field, so it routed through the generic `extras` bag, and Ortie host-matched `api.fastmail.com` to pin the resource and a scope set. Fastmail publishes both: its JMAP session and DAV roots answer RFC 9728 protected resource metadata.

## What
A discovery mechanism belongs to whichever library owns the RFC that defines it, and the resource fix is typed in three halves:

- io-oauth (change resource-indicators): `resource: Vec<Url>` on `Oauth20AuthRequestParams` and `Oauth20AccessTokenRequestParams`, one `resource=` pair each.
- io-pim-discovery (change oauth-resource): the OAuth grants carry `resource` and `scopes_supported`, filled from the RFC 9728 metadata of the service endpoint and the RFC 8414 metadata of the issuer.
- Ortie: an account `resources` list sent on the authorization request and the code exchange; the wizard writes the chosen grant's resource and preselects the scopes that resource advertises. `fill_provider_defaults` and the Fastmail scope list are gone.

A grant merged across services keeps one resource, the first in compose priority order (the JMAP session for Fastmail): whether a server accepts several indicators on one request is unverified, Fastmail's authorize endpoint checking them in its browser app only.

Moving the RFC 8414 and 9728 types and their fetch coroutines into io-oauth is no longer needed to unblock anything and became its own draft, io-pim-discovery's oauth-metadata-relocation.
