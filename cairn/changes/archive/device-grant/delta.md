---
cairn: delta
change: device-grant
---

## ADDED Requirements

### Requirement: Device authorization endpoint
An account SHALL accept `endpoints.device-authorization`, checked by `auth get` only on a device account.

### Requirement: Device grant runs
On a device account, `auth get` SHALL request device authorization, display the user code and verification URI (preferring `verification_uri_complete` when present), poll to completion via io-oauth `await_device_access_token`, write storage, and fire the on-issue hooks. `auth resume` SHALL interpret its positional as the device code, and the authorization-code-only flags SHALL be rejected.

## REMOVED Requirements

### Requirement: Device grant parses but does not run
