---
cairn: tasks
change: deny-warnings-cleared
---

- [x] deny.toml: `[graph] targets`, the getrandom 0.2 skip, CDLA-Permissive-2.0 dropped
- [x] pimalaya-cli, pimalaya-config and io-oauth changed and checked, uncommitted
- [x] Ortie checked against the three local checkouts: deny with no duplicate warning, 104 crates, tests pass, 4,336,536 bytes
- [x] Commit and push pimalaya-cli, pimalaya-config and io-oauth
- [x] Cargo.toml: pin the three by git rev in `[patch.crates-io]`, `cargo update -p` them
- [x] deny.toml: drop the windows-sys 0.52 skip, matched by nothing once the lock re-resolves
- [x] `cargo deny check` with no warning, clippy, tests, `ortie --version` naming the commit
- [x] Delta folded, log written, status landed
