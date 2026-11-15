---
cairn: tasks
change: token-revocation
---

- [x] io-oauth: revocation request coroutine + `Oauth20ClientStd` method (io-oauth change token-revocation)
- [x] Add optional `endpoints.revocation` config field + `Account` counterpart
- [x] Prefill `endpoints.revocation` in the wizard from metadata `revocation_endpoint`
- [x] Add `token revoke` (refresh token when present, else access token); storage left as is, see the proposal
- [x] Update `config.sample.toml`, README and CHANGELOG
