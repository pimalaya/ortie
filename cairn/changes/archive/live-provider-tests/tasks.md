---
cairn: tasks
change: live-provider-tests
---

- [x] ~~Land io-oauth's `live-provider-tests` first~~ Dropped: a failing Ortie test that reaches a provider already names the provider's error.
- [x] tests/common.rs: shared config writer, `ortie()` runner, TempDir storage, env helper
- [x] tests/microsoft.rs `anonymous::`: device `auth get --json`, client credentials rejected
- [x] tests/microsoft.rs `credentialed::`: client credentials acquire, reuse, re-acquire; JWT kind; renewal hint
- [x] The JWT kind run live, by the owner: a certificate uploaded to the Pimalaya registration, its key and certificate as the `MICROSOFT_CLIENT_KEY` and `MICROSOFT_CLIENT_CERT` repository secrets
- [x] "Allow public client flows" turned on for the Pimalaya Microsoft client, by the owner, so `device_auth_get_json` passes
- [x] tests/google.rs `anonymous::`: `auth resume` and `token revoke` rejections
- [x] tests/fastmail.rs `anonymous::`: `auth resume` rejection
- [x] .github/workflows/tests.yml: per-provider jobs, credentialed steps repository-guarded, `RUST_LOG=debug`
- [x] Fold the delta into spec/packaging.md, log, archive
