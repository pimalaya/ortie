---
cairn: tasks
change: token-revocation
---

- [ ] io-oauth: revocation request coroutine + `Oauth20ClientStd` method
- [ ] Add optional `endpoints.revocation` config field + `Account` counterpart
- [ ] Prefill `endpoints.revocation` in the wizard from metadata `revocation_endpoint`
- [ ] Add `token revoke` (refresh token when present, else access token; clear/overwrite storage after)
- [ ] Update `config.sample.toml`, README and CHANGELOG
