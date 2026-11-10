---
cairn: change
id: config-flag-repeated
status: landed
created: 2026-11-10
---

# Config flag repeated

## Why

`-c` and `ORTIE_CONFIG` split their value on `:`, so a Windows path such as `C:\Users\me\config.toml` was cut at the drive letter into `C` and `\Users\me\config.toml`, repeated `-c` included. Himalaya hit it in pimalaya/himalaya#800 and moved to the platform's `$PATH` separator. The delimiter only came in for the environment variable (pimalaya/himalaya#612), `-c` inheriting it because clap splits a flag and its `env` value alike. The owner chose on 2026-11-10 to drop both the delimiter and the variable: a repeated `-c` says the same thing with nothing to explain, and a shell variable still reaches it as `-c "$VAR"`.

## What

- `-c` takes one path per occurrence, never split, repeated to merge several files in order.
- `ORTIE_CONFIG` is gone, and clap's `env` feature with it (Ortie was its only user in the build).
- The `auth listen` and `auth resume` commands `auth get` prints carry one `-c` per path instead of a `:`-joined list.
- README, `--config` help, CHANGELOG and spec say so. The Pimalaya guidelines take it as the family default: a new cli-009 rule and the readme-010 template (owner, 2026-11-10).
