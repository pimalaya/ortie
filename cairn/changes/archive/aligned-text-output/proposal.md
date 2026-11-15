---
cairn: change
id: aligned-text-output
status: landed
created: 2026-11-11
---

# Aligned text output

## Why

`token inspect` printed `Token type: bearer`, `With refresh token: true` and every scope on one line, and `auth get` listed its requests as ` - state: ...` bullets with its commands behind a `> ` prompt, while the welcome indents its commands two spaces. The owner asked on 2026-11-10 for a standard way to print plain-text data and on 2026-11-11 for a shared helper; pimalaya-cli gained `fields::Fields` (its change aligned-fields, 5dd9416).

## What

- Ortie pins pimalaya-cli 5dd9416.
- `token inspect` prints `Type`, `Issued`, `Expires` (`in <duration>`, `expired` or `unknown`), `Refresh token` (`yes` or `no`) and `Scopes`, one per line, through `Fields`; the token type prints as the server spelled it.
- `auth get` prints the authorization request (`State`, `PKCE verifier`, `Redirect URI`) and the device request (`User code`, `Verification URI`, `Complete URI`, `Device code` when handed off, `Expires in`, `Polling interval`) through `Fields`, then the link to open after a blank line, indented two spaces.
- The `auth listen` and `auth resume` commands `auth get` prints lose their `> ` prompt for a two-space indent.
