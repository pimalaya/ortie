---
cairn: change
id: device-grant
status: landed
created: 2026-07-25
---

# Device authorization grant end-to-end

## Why
`grant = "device"` already parses but `auth get`/`auth resume` bail on it. The device grant (RFC 8628) is one of the grants OAuth 2.1 keeps, and io-oauth already ships everything needed (`await_device_access_token`); the gap is purely Ortie's config field and CLI wiring.

## What
Make a device account runnable. Add the `endpoints.device-authorization` config field (deferred from M1) and its `Account` counterpart. `auth get` dispatches on the configured grant: request device authorization, display the user code and verification URI (preferring `verification_uri_complete`), poll to completion, write storage, and fire the on-issue hooks shared with the code-grant path. `auth resume` interprets its positional per the account's grant (redirected URI for authorization code, device code for device), and the authorization-code-only flags (`--state`/`--pkce`/`--redirect-uri`) are rejected on device accounts. No io-oauth work needed.

## Deferred: a resume command per grant
One `auth resume` serves both grants via a shared positional (`URI|DEVICE_CODE`), with `--state`/`--pkce`/`--redirect-uri` code-grant-only and rejected on device accounts. That asymmetry is inherent, not accidental: `state` and PKCE only protect the authorization-code front-channel redirect (CSRF on the callback, and interception of the code in transit), which the device grant has none of, so it carries no such round-trip state and needs only the device code. Splitting `resume` (and, less usefully, the mixed generic/specific endpoint fields) into a cleaner per-grant shape is possible, but it is a breaking change to the CLI surface. Since v2.0.0 just published, it is put aside rather than forced into an early major bump.
