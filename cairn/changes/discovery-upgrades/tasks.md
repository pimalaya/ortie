---
cairn: tasks
change: discovery-upgrades
---

- [ ] Enable io-pim-discovery's rfc9728 feature
- [ ] Resolve RFC 8414 issuer metadata on `://` inputs, fall back to RFC 9728 then manual (pre-seeded)
- [ ] Drive offered grants from `grant_types_supported` + `device_authorization_endpoint`
- [ ] Drive the suggested `pkce` from `code_challenge_methods_supported`
- [ ] Retry metadata resolution for unresolvable bare issuer picks
