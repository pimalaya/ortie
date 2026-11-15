---
cairn: delta
change: release-polish
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Refresh rotation ordering
On refresh, Ortie SHALL persist the new token to storage as soon as the server answers, and SHALL keep the previous refresh token when the server omits a rotated one. The new token SHALL be held in memory before the write, so a failed write leaves the running session on the issued tokens, and the failure SHALL say that the next refresh may need `auth get`. A refresh failure fires the on-refresh error hook and reports the server error code.

## REMOVED Requirements
