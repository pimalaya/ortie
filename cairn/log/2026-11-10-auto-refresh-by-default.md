---
cairn: log
change: auto-refresh-by-default
landed: 2026-11-10
---

# Auto-refresh by default

`AccountConfig::auto_refresh` (src/config.rs) defaults to true through `auto_refresh_default`, so `token show` refreshes an expired token, or re-acquires it on the client credentials grants, unless the account sets `auto-refresh = false`. `token show --auto-refresh` is unchanged and now only matters for such an account, as its help says. config.sample.toml keeps `auto-refresh = true` active, the default, with `#auto-refresh = false` under it, which the sample test parses. A `no-auto-refresh` rename was proposed and left: a negative key, and a break for every account setting `auto-refresh`, unknown keys being refused.

`auto_refresh_is_on_unless_turned_off` parses an account without the key and one turning it off; it failed with the default flipped back to false.

Capabilities moved: token (cairn/spec/token.md: "Auto-refresh on show" names the default), config (cairn/spec/config.md: "v1 config compatibility" names the refresh as a behaviour an existing account notices).
