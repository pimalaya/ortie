---
cairn: log
change: token-revocation
date: 2026-11-03
---

# Token revocation

`token revoke` (src/token/revoke.rs) revokes the stored refresh token, or the access token when none is stored, against the new optional `endpoints.revocation`, through io-oauth's `Oauth20ClientStd::revoke_token` with the matching type hint, and prints `{revoked}` (`refreshToken` or `accessToken`), registered as `ortie-token-revoke`. A rejection fails with the server's code. Storage is left untouched, the original plan's clearing being impossible with a write-only storage command fed a token JSON. The wizard writes the metadata's `revocation_endpoint` as `endpoints.revocation`.

Capabilities moved: `token` (Revoke command added), `config` (Late-bound endpoints modified), `discovery` (Revocation endpoint prefilled added), `output` (the `ortie-token-revoke` key).

Verified: tests/token_revoke.rs drives the binary against a mock endpoint (refresh token revoked with storage kept, access token revoked without one, a rejection failing with its code); the registry test covers the new key; a fragment carrying `endpoints.revocation` parses back. No live revocation was run.
