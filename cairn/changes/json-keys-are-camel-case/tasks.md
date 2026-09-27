---
cairn: tasks
change: json-keys-are-camel-case
---

Held until the 3.0 cycle opens.

- [ ] `#[serde(rename_all = "camelCase")]` on the four registered types and the types they carry: `ConfigureOutput`, `Endpoints`, `TokenShowOutput`, `TokenInspectOutput`, `AuthGetOutput` with `AuthorizationUri` and `DeviceAuthorization`
- [ ] Drop the kebab-case rename from `ConfigureOutput` and `Endpoints`, and note in their docs that the rendered TOML document stays hyphenated
- [ ] Leave src/config.rs kebab-case, and leave any provider passthrough `#[serde(rename)]` untouched
- [ ] Regenerate the JSON Schemas and check no key kept a hyphen or an underscore
- [ ] CHANGELOG under `Changed`, as breaking, in the 3.0 section
