---
cairn: log
change: live-provider-tests
landed: 2026-11-11
---

# Live provider tests

tests/google.rs, tests/microsoft.rs and tests/fastmail.rs run the binary against the real providers, every test ignored unless `--ignored` is given. tests/common.rs holds what the suites share: `write_config` (one default account over `cat` and `tee` storage in a TempDir), `ortie` and `ortie_with_input`, `stored`, `json_failure`, `sample_section` (a provider section of config.sample.toml, uncommented, keys overridable), `env_var`, `require_env` and `env_file` (an empty variable counts as unset, `_FILE` names a file), and the fixture key. The six mock suites moved onto it.

- `anonymous` (network only): Microsoft `device_auth_get_json` on the sample's Graph section with a device grant, `client_credentials_rejected` (the public client, a made-up secret: exit 1, `InvalidClient`); Google `auth_resume_rejected` (`InvalidClient`) and `token_revoke_rejected` (exit 1, storage kept) on the sample's Google section with a made-up client; Fastmail `auth_resume_rejected` on the sample's section, its description surfaced.
- `credentialed` (Microsoft, the Pimalaya registration of the test tenant as a confidential client): `client_credentials_token_show` acquires from empty storage, serves the stored token without writing it again, and re-acquires an expired one; `client_credentials_jwt_token_show` acquires and mints again from `ORTIE_MICROSOFT_CLIENT_KEY` and `ORTIE_MICROSOFT_CLIENT_CERT`; `client_credentials_jwt_renewal_hint` signs with the fixture key and an unregistered certificate and gets `InvalidClient` (AADSTS700027) with the renewal hint. The secret reaches the binary through `printenv`, the key and certificate through files only the owner reads, and a failure shows stderr and the exit code, never stdout.

.github/workflows/tests.yml gains a `live` job per provider (a matrix, `fail-fast: false`) at `RUST_LOG=debug`, whose logs were checked to carry neither the secret nor the token. The credentialed steps run in pimalaya/ortie only, mapping each `MICROSOFT_*` secret to its `ORTIE_MICROSOFT_*` variable, and the JWT step only once `MICROSOFT_CLIENT_KEY` exists. io-oauth's suites were not landed first.

Every test passed on 2026-11-11 from this machine, after two fixes on the registration the first run asked for. `device_auth_get_json` failed with AADSTS70002 ("The client application must be marked as 'mobile'") until the owner turned on its public client flows, which the device grant needs. The JWT kind runs on a self-signed certificate (CN "Pimalaya live tests", SHA-1 thumbprint E7A1E62D9B13680AA39925094EA41DCC5EC3B62B, expiring 2028-11-10) the owner uploaded, its key and certificate held by the `MICROSOFT_CLIENT_KEY` and `MICROSOFT_CLIENT_CERT` repository secrets; once it expires, the JWT step fails with the renewal hint. Google answers a made-up revocation with `invalid_token`, outside RFC 7009's codes, which io-oauth reads as `Unknown`, so the error loses Google's code.

Capabilities moved: packaging (cairn/spec/packaging.md: "Live provider suites" added).
