---
cairn: log
change: device-grant
landed: 2026-07-25
---

# Device authorization grant (RFC 8628)

Landed the M2 device authorization grant from a community pull request by Rohit Goswami (`rgoswami@ieee.org`), applied by hand on top of the persistent-repl changes. Added `endpoints.device-authorization` to `EndpointsConfig` and `device_authorization_endpoint` on `Account`. `auth get` now dispatches on `account.grant`: the authorization-code path is unchanged; the device path calls io-oauth `request_device_auth`, displays the user code and verification URI (preferring `verification_uri_complete`), and either polls in-process via `await_device_access_token` (interactive) or prints the device response and hands off (non-interactive / `--json`), sharing the storage write and on-issue hooks with the code grant through `report_token_issued`. `auth resume`'s positional became `URI|DEVICE_CODE`, grant-interpreted; a bare device code polls with the RFC 8628 example defaults (`expires_in = 1800`, `interval = 5`), and the authorization-code-only flags are rejected on device accounts. The `grant = "device"` bail placeholders are gone. A local device-poll deadline fires the on-issue error hook as `expired_token`, printed `auth resume` examples shell-quote secrets, and resume errors omit redirect/state/PKCE bodies. Reconciled with the local changes: `report_token_issued` clones into the owned `write_to_storage`, which stamps `issued_at` locally.

Spec: `auth` (REMOVED "Device grant parses but does not run"; ADDED device authorization endpoint and device grant runs), `provider-quirks` (Microsoft device endpoints). Deviation: account `extras` are not forwarded on the device authorization request (`Oauth20DeviceAuthRequestParams` has no extras table in io-oauth).
