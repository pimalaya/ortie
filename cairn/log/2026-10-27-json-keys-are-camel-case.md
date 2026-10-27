---
cairn: log
change: json-keys-are-camel-case
landed: 2026-10-27
---

# `--json` keys are camelCase

Every output type now carries `#[serde(rename_all = "camelCase")]`: `TokenShowOutput` prints `accessToken`, `TokenInspectOutput` `tokenType`, `issuedAt`, `expiresIn` and `withRefreshToken`, the `AuthGetOutput` variants `authorizationUri`, `pkceCodeVerifier`, `deviceCode`, `userCode`, `verificationUri`, `verificationUriComplete` and `expiresIn`, and `ConfigureOutput` `clientId`, `clientSecret`, `autoRefresh` and `endpoints.deviceAuthorization`, both losing their kebab-case rename. `RawSecret`, `Storage` and `StorageEntry` hold single-word keys today and take the rename too, so a later field cannot slip back to snake_case. The untagged `AuthGetOutput` has no keys of its own.

The proposal spoke of a TOML `document` field inside the `configure --json` object. There is none: the TOML fragment is the text rendering, hand-written by `ConfigureOutput`'s `Display`, so it stays kebab-case untouched and the delta now says so. src/config.rs stays kebab-case. The stored token file is io-oauth's wire format, not an output type, and keeps `access_token`.

The regenerated schemas hold no key with a hyphen or an underscore. tests/device_grant.rs reads `deviceCode`.

The proposal held this for 3.0; it lands now under [Unreleased] as breaking, Ortie still being 2.3.0, so the `$id` of each schema says `v2` until the version moves to 3.

Spec updated: output (ADDED: JSON keys are camelCase).
