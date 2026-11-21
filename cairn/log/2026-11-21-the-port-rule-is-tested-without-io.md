---
cairn: log
change: the-port-rule-is-tested-without-io
landed: 2026-11-21
---

# The port rule is tested without I/O

The nixpkgs review of ortie 3.0.0 (NixOS/nixpkgs#591248) failed on aarch64-darwin: two unit tests of src/account.rs called `bind_loopback`, which binds a real socket, and the macOS build sandbox refuses it (`PermissionDenied`, and `localhost` not resolving). Linux passed only because its sandbox keeps loopback.

## What landed

- src/account.rs: `bind_loopback` keeps the binding and calls two pure functions, `loopback_port` (the port a redirection URI names, 0 for an ephemeral one on an http URI naming none, else its scheme's known default) and `with_bound_port` (the URI carrying the bound port when an ephemeral one was asked for).
- The two socket tests are replaced by five tests of those functions, written first and failing on the previous tree (unresolved imports). Binding itself stays covered by the tests/ suites, which `auth get`, `auth listen` and MCP `auth_get` exercise end to end.

## Spec

No capability moved: the port rule is unchanged.

## Verified

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (143 passed), and the unit test binary run in a network namespace with loopback down (`unshare -rn`): 41 passed.
