---
cairn: change
id: device-grant
status: draft
created: 2026-07-25
---

# Device authorization grant end-to-end

## Why
`grant = "device"` already parses but `auth get`/`auth resume` bail on it. The device grant (RFC 8628) is one of the grants OAuth 2.1 keeps, and io-oauth already ships everything needed (`await_device_access_token`); the gap is purely Ortie's config field and CLI wiring.

## What
Make a device account runnable. Add the `endpoints.device-authorization` config field (deferred from M1) and its `Account` counterpart. `auth get` dispatches on the configured grant: request device authorization, display the user code and verification URI (preferring `verification_uri_complete`), poll to completion, write storage, and fire the on-issue hooks shared with the code-grant path. `auth resume` interprets its positional per the account's grant (redirected URI for authorization code, device code for device), and the authorization-code-only flags (`--state`/`--pkce`/`--redirect-uri`) are rejected on device accounts. No io-oauth work needed.
