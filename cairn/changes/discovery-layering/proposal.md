---
cairn: change
id: discovery-layering
status: draft
created: 2026-07-25
---

# Discovery layering: typed resource and spec-provenance

## Why
Fastmail refused to work in Ortie for the same reason it once refused in cardamum-android: both consumers had to re-derive the same provider knowledge by hand. Fastmail's authorize endpoint hard-requires an RFC 8707 `resource` parameter; io-oauth models the *failure* (`InvalidTarget`, doc even names Fastmail) but no request type carries a `resource` field, so it routes through the generic `extras` bag and every consumer re-learns the trap. This change spans io-oauth and io-pim-discovery; Ortie is the consumer that surfaced it.

## What
Adopt a boundary rule — a discovery mechanism belongs to whichever library owns the RFC that defines it (spec-defined discovery in the domain library, heuristic/cross-domain in io-pim-discovery) — and make the resource fix typed in two halves:

- io-oauth gains a first-class `resource: Vec<Url>` on `Oauth20AuthRequestParams` and `Oauth20AccessTokenRequestParams` (RFC 8707 allows several; empty omits it), serialized as repeated `resource=` pairs and documented next to the `InvalidTarget` it prevents.
- io-pim-discovery carries `resource: Option<Url>` on the OAuth grant descriptors, filled from the RFC 9728 metadata during the refine pass, so consumers do trivial typed wiring (`auth_params.resource = grant.resource`).

The same treatment applies less urgently to scope negotiation (carry advertised `scopes_supported` on the grant). The larger move — relocating the RFC 8414/9728 types and their pure fetch coroutines back into io-oauth, leaving only orchestration in io-pim-discovery — sequences OAuth first (trivial pure-HTTP, unblocks the resource trap), then the SRV-heavy JMAP/DAV discovery once a shared DNS/SRV want exists; autoconfig, ISPDB and provider rules never move.

## Stopgap in effect today
None of the above is needed to use Fastmail now. Ortie forwards `extras` verbatim, so a Fastmail account works the moment its config carries the resource by hand (`extras.resource = "https://api.fastmail.com/jmap/session"`), and `fill_provider_defaults` in `discover.rs` host-matches `api.fastmail.com` to pin it (and a default scope set when discovery yielded none). The typed fix removes the host-match table; it does not unblock anything currently blocked.
