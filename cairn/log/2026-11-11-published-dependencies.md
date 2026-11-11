---
cairn: log
id: published-dependencies
date: 2026-11-11
---

# Build on the published dependencies

io-oauth 0.5.2, pimalaya-cli 0.3.2 and pimalaya-config 0.3.1 were published on 2026-11-11, carrying every commit Ortie pinned: io-oauth 1f88ccb (independent-random-draws, rsa 0.10.0), pimalaya-cli 5dd9416 (machine-output-contract, logs-and-help-stay-plain, one-dirs-version, build-scripts-run-git, aligned-fields) and pimalaya-config 7601a55 (one-dirs-version, notification templates). Cargo.toml drops its `[patch.crates-io]` section and requires those versions as minimums, `fields::Fields` existing only from pimalaya-cli 0.3.2; Cargo.lock moves the three from git to the registry and nothing else.

deny.toml keeps `allow-org.github = ["pimalaya"]` for the next git pin, so `cargo deny check` passes with two warnings: that allowance going unmatched, and the getrandom duplicate packaging.md keeps visible.
