---
cairn: tasks
change: discovery-upgrades
---

- [x] Enable io-pim-discovery's rfc9728 feature (landed with discovery-layering)
- [x] Resolve RFC 8414 metadata on `://` inputs, falling back to RFC 9728 then its authorization servers; the manual fallback is obsolete, the wizard stops on nothing discovered
- [x] Drive offered grants from `grant_types_supported` and `device_authorization_endpoint` (endpoints since 2.1.0, grant types now)
- [x] Drive the suggested `pkce` from `code_challenge_methods_supported`
- [x] Retry metadata resolution for unresolvable bare issuer picks: already done by the wizard's own issuer pass, nothing left
