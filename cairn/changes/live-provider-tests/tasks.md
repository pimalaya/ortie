---
cairn: tasks
change: live-provider-tests
---

- [ ] Land io-oauth's `live-provider-tests` first
- [ ] tests/common.rs: shared config writer, `ortie()` runner, TempDir storage, env helper
- [ ] tests/microsoft.rs `anonymous::`: device `auth get --json`, client credentials rejected
- [ ] tests/microsoft.rs `credentialed::`: client credentials acquire, reuse, re-acquire; JWT kind; renewal hint
- [ ] tests/google.rs `anonymous::`: `auth resume` and `token revoke` rejections
- [ ] tests/fastmail.rs `anonymous::`: `auth resume` rejection
- [ ] .github/workflows/tests.yml: per-provider jobs, credentialed steps repository-guarded, `RUST_LOG=debug`
- [ ] Fold the delta into spec/packaging.md, log, archive
