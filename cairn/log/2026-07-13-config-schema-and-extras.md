---
cairn: log
change: config-schema-and-extras
landed: 2026-07-13
---

# M1: config schema and extras wiring

Added the flat `grant` selector (`GrantConfig`, default authorization-code); `grant = "device"` parses but `auth get`/`auth resume` bail until the device grant lands. Made `endpoints.authorization`/`token`/`redirection` optional at parse time, with each command checking the endpoints it needs (`auth get` → authorization, `auth resume`/`token refresh` → token, `token show` → none). Widened `pkce` to bool-or-string via a hand-rolled serde visitor (`true`/`"s256"` = S256, `"plain"`, `false` = off), with the omitted default now S256. Added the `extras` account table (keys verbatim, no kebab renaming), wired into the authorization request query via io-oauth `Oauth20AuthRequestParams.extras`. Reshaped `auth discover` output into a paste-ready fragment. Updated `config.sample.toml`, README and CHANGELOG.

Deviation: `endpoints.device-authorization` moved to the device-grant change, since landing it without its reader would leave a dead-code field. Context: io-oauth stabilised at 0.1.0 with a per-RFC layout mid-milestone, so Ortie builds against the published crate with no local patch.

Spec: `config` (grant selector, late-bound endpoints, PKCE shape, extras, v1 compatibility), `auth` (device parses but does not run).
