---
cairn: tasks
change: discovery-layering
---

- [x] io-oauth: add typed `resource: Vec<Url>` to `Oauth20AuthRequestParams` and `Oauth20AccessTokenRequestParams` (repeated `resource=`), documented next to `InvalidTarget` (io-oauth change resource-indicators)
- [x] io-pim-discovery: carry `resource` on OAuth grant descriptors, filled from RFC 9728 metadata (io-pim-discovery change oauth-resource)
- [x] Ortie: send the account `resources`; the wizard writes `grant.resource`; retire the `fill_provider_defaults` host match
- [x] Carry advertised `scopes_supported` on the grant so consumers stop hand-rolling scope lists
- [x] (Larger) relocating the RFC 8414/9728 types into io-oauth: split out as io-pim-discovery's draft change oauth-metadata-relocation
