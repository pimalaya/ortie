---
cairn: change
id: release-polish
status: draft
created: 2026-07-25
---

# Release polish

## Why
Before releasing the minor that carries the device grant, the OAuth 2.1 posture should be verified and documented, and the refresh rotation window audited so a rotated refresh token can never be lost.

## What
Audit the write-after-refresh window in `token refresh` to confirm a rotated refresh token is persisted before the old one can be lost. Add a README section on the 2.1 posture (grants supported, PKCE default, exact redirect matching with the loopback variable-port exception, rotation behaviour). Add CHANGELOG notes (pkce default flip, discover output shape, lib removal; no config migration). Re-check the final RFC if published and release the minor. This is verification and documentation only; the rotation-ordering behaviour it verifies is already spec (`token`).
