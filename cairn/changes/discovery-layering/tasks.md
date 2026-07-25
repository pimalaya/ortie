---
cairn: tasks
change: discovery-layering
---

- [ ] io-oauth: add typed `resource: Vec<Url>` to `Oauth20AuthRequestParams` and `Oauth20AccessTokenRequestParams` (repeated `resource=`), documented next to `InvalidTarget`
- [ ] io-pim-discovery: carry `resource: Option<Url>` on OAuth grant descriptors, filled from RFC 9728 metadata in the refine pass
- [ ] Ortie: wire `auth_params.resource = grant.resource`; retire the `fill_provider_defaults` host-match for Fastmail resource
- [ ] Carry advertised `scopes_supported` on the grant so consumers stop hand-rolling scope intersections
- [ ] (Larger) relocate RFC 8414/9728 types + pure fetch coroutines into io-oauth; add a shared DNS/SRV want before moving JMAP/DAV discovery
