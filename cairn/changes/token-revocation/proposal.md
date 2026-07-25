---
cairn: change
id: token-revocation
status: draft
created: 2026-07-25
---

# Token revocation (RFC 7009)

## Why
Ortie can issue and refresh tokens but not revoke them. Revocation rounds out the token lifecycle and is a natural companion to the wizard's metadata-driven discovery (the `revocation_endpoint` is advertised alongside the rest).

## What
io-oauth first: a revocation request coroutine plus an `Oauth20ClientStd` method. Then Ortie: an optional `endpoints.revocation` (prefilled by the wizard from the metadata `revocation_endpoint`) and a `token revoke` command that revokes the refresh token when present, else the access token (per the RFC 7009 recommendation), clearing or overwriting storage afterwards.
