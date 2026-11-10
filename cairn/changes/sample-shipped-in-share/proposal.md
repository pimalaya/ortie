---
cairn: change
id: sample-shipped-in-share
status: landed
created: 2026-11-10
---

# Sample shipped in share

## Why

The welcome downloaded config.sample.toml from the master branch, which needs the network and can be ahead of the installed release. Samples usually ship with the release, beside the man pages under share/. The owner asked on 2026-11-10 for the welcome to take the sample from the local share first, and from the network only without one, and whether its commands work on Windows: they did not, single quotes being no quotes to cmd and `curl` being Invoke-WebRequest in Windows PowerShell 5.1.

## What

- package.nix installs config.sample.toml as share/doc/ortie/config.sample.toml, so the Nix package and every release archive (built from it, with share/ beside the binary) carry it; install.sh copies it under its prefix when the archive has it.
- `shipped_sample` (src/cli.rs) looks for it under the prefix holding the binary's directory, then under that directory, the extracted archive layout; the binary is canonicalized first on Unix so a Nix profile or Homebrew symlink leads to its prefix.
- The welcome takes it: `mkdir -p` for a missing directory, then `install -m 644`, since `cp` would keep the read-only mode of the Nix store. Without one it downloads it as before.
- `shell_quote` (src/auth/get.rs, formerly `shell_single_quote`) double quotes on Windows, and the welcome uses `curl.exe`, `mkdir` and `copy` there, which cmd and PowerShell both run; the `auth get` handoff commands are quoted the same way.

## Out of scope

The network fallback still fetches master's sample, which can differ from the installed version; pinning it to the version tag would break builds of unreleased versions.
