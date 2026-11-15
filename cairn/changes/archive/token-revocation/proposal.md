---
cairn: change
id: token-revocation
status: landed
created: 2026-07-25
---

# Token revocation (RFC 7009)

## Why
Ortie can issue and refresh tokens but not revoke them. Revocation rounds out the token lifecycle and is a natural companion to the wizard's metadata-driven discovery (the `revocation_endpoint` is advertised alongside the rest).

## What
io-oauth first (its change token-revocation): the `rfc7009::revoke` coroutine and `Oauth20ClientStd::revoke_token`. Then Ortie: an optional `endpoints.revocation`, prefilled by the wizard from the metadata `revocation_endpoint`, and a `token revoke` command that revokes the refresh token when present, else the access token (the RFC 7009 recommendation), printing which one went.

Adjusted from the original plan, which cleared or overwrote storage afterwards: Ortie only has a write command, fed a token JSON, so clearing would store a value `token show` cannot parse. Storage is left as it is; the revoked token no longer works, and the next `auth get` replaces it.
