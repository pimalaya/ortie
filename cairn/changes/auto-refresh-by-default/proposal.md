---
cairn: change
id: auto-refresh-by-default
status: landed
created: 2026-11-10
---

# Auto-refresh by default

## Why

`token show` printed an expired token unless the account set `auto-refresh = true` or the call passed `--auto-refresh`, while what calls it (a mail client, a sync daemon) wants a token it can use; the sample already set it to true. The owner proposed on 2026-11-10 to default it to true, or to rename it `no-auto-refresh`. The rename was left: a negative key reads as `no-auto-refresh = false`, and unknown keys being refused, it would break every account setting `auto-refresh`.

## What

- `auto-refresh` defaults to true (src/config.rs); `auto-refresh = false` turns it off.
- `token show --auto-refresh` stays, forcing it on an account turning it off; its help says so.
- config.sample.toml shows `#auto-refresh = false` under the active default.
- Breaking for an account that relied on the old default: an expired token is now refreshed, or re-acquired on the client credentials grants, on `token show`.
