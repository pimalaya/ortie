---
cairn: delta
change: token-revocation
---

## ADDED Requirements

### Requirement: Revocation endpoint
An account MAY carry an optional `endpoints.revocation`, prefilled by the wizard from the metadata `revocation_endpoint`.

### Requirement: Revoke command
`token revoke` SHALL revoke the refresh token when present, else the access token (per RFC 7009), and clear or overwrite storage afterwards.
