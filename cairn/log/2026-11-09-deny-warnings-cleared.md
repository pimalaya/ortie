---
cairn: log
change: deny-warnings-cleared
landed: 2026-11-09
---

# Deny warnings cleared

`cargo deny check` passes with no warning. Ortie pins, through `[patch.crates-io]`, pimalaya-cli ce008cc (one-dirs-version and build-scripts-run-git: paths expand on dirs 7, and `git_envs` runs the git command, git2 gone), pimalaya-config 7601a55 (one-dirs-version: the tilde expands on dirs 7) and io-oauth 52ef73b (independent-random-draws: the state and PKCE verifier draw every character independently from getrandom 0.4, rand gone). dirs 6, rand, chacha20, git2, libgit2-sys and libz-sys leave the lock.

deny.toml lists the targets Ortie ships or is built for under `[graph] targets`, so Android's jni crates and their syn 2 leave every check, and CDLA-Permissive-2.0, which only webpki-root-certs on other platforms carried, leaves the allowed licences. getrandom 0.2 is skipped with ring 0.17 as the reason. The windows-sys 0.52 skip was dropped once the lock re-resolved: on the listed targets nothing uses it any more, ring needing it on aarch64 Windows only.

Capabilities moved: packaging (cairn/spec/packaging.md: "Dependency checks pass clean" added).

Verified: `cargo deny check` (no warning), `cargo fmt --check`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`, `cargo check --no-default-features --features native-tls`, `ortie --version` naming the commit, `nix eval` of the package. 104 crates in the normal tree (107 before); a release build against the same three checkouts weighed 4,336,536 bytes (4,342,976 before).
