---
cairn: change
id: deny-warnings-cleared
status: landed
created: 2026-11-09
---

# Deny warnings cleared

## Why

After dependencies-trimmed (2026-11-09), `cargo deny check` still warned about four duplicate crates:

- dirs 6 and 7: shellexpand's `tilde` feature, which pimalaya-cli and pimalaya-config turned on, pins dirs below 7.
- syn 2 and 3: syn 2 comes only from jni-macros and jni-sys-macros, which rustls-platform-verifier pulls for Android, and cargo deny checked every target.
- getrandom 0.2 and 0.4: ring 0.17 needs 0.2; io-oauth (through rand), tempfile and cc's jobserver need 0.4.
- windows-sys 0.52 and 0.61: rustls-platform-verifier and tempfile resolved to 0.52 alongside ring.

The same review found two more things. pimalaya-cli's `build` feature compiled libgit2 and zlib from C to read the commit for `--version`. io-oauth drew its state and PKCE verifier without repetition, capping a verifier at 66 characters and a state at 95.

## What

- **pimalaya-cli** (one-dirs-version, build-scripts-run-git): paths expand through `shellexpand::full_with_context` on dirs 7, and `git_envs` runs the git command against the working directory's own repository, falling back to `unknown`, with git2 gone.
- **pimalaya-config** (one-dirs-version): the same expansion on its own dirs 7.
- **io-oauth** (independent-random-draws): getrandom 0.4 replaces rand, and every character is drawn independently with rejection sampling.
- **Ortie**: pins the three by `[patch.crates-io]`: pimalaya-cli ce008cc, pimalaya-config 7601a55, io-oauth 52ef73b. deny.toml gets `[graph] targets` (the release targets plus the Linux gnu and Windows msvc ones people build from source), which drops the Android-only crates, syn 2 and the CDLA-Permissive-2.0 allowance. It skips getrandom 0.2 with ring 0.17 named as the reason.

## Not in this change

- ring moving off getrandom 0.2: upstream.
