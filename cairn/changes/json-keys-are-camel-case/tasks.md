---
cairn: tasks
change: json-keys-are-camel-case
---

- [x] `#[serde(rename_all = "camelCase")]` on the four registered types and the types they carry: `ConfigureOutput`, `Endpoints`, `TokenShowOutput`, `TokenInspectOutput`, `AuthGetOutput` with `AuthorizationUri` and `DeviceAuthorization` (the untagged `AuthGetOutput` itself has no keys, its variant payloads carry the rename)
- [x] Drop the kebab-case rename from `ConfigureOutput` and `Endpoints`, and note in their docs that the rendered TOML document stays hyphenated
- [x] Leave src/config.rs kebab-case, and leave any provider passthrough `#[serde(rename)]` untouched
- [x] Regenerate the JSON Schemas and check no key kept a hyphen or an underscore
- [x] CHANGELOG under `Changed`, as breaking, in [Unreleased] (no 3.0 section exists yet)
