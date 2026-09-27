---
cairn: change
id: json-keys-are-camel-case
status: draft
created: 2026-08-29
---

# `--json` keys become camelCase at 3.0

## Why

The Pimalaya family standardises the object keys of `--json` on camelCase. Ortie is 2.2.0, its `--json` payloads are a published contract, and renaming a key is breaking, so it switches at 3.0 rather than now. This entry records the decision so the next major does not have to rediscover it.

camelCase is what the formats around these tools already speak: JMAP objects are camelCase by RFC 8620, and so are Microsoft Graph and the Google APIs. OAuth itself is the exception that proves nothing, being snake_case on the wire (`access_token`, `expires_in`), but Ortie prints its own types rather than the wire ones, and the family is one surface to a script.

The stronger reason is the consumer `--json` exists for. Neither JavaScript nor jq can use dot access on a key holding a hyphen: `.client-id` has to be written `."client-id"` in jq and `obj["client-id"]` in JavaScript, and the unquoted form fails quietly. `clientId` reads the same in both.

Ortie emits both conventions today. `ConfigureOutput` and its `Endpoints` (src/wizard.rs) carry `#[serde(rename_all = "kebab-case")]` and print `client-id`, `client-secret`, `device-authorization`. The other three registered types carry no rename and print serde's snake_case: `access_token` from `TokenShowOutput`, `token_type` and `with_refresh_token` from `TokenInspectOutput`, `authorization_uri` and `pkce_code_verifier` from the `AuthGetOutput` variants.

## What changes at 3.0

Output types only, meaning the four types registered in src/json_schema.rs and the types they carry:

- `ConfigureOutput` and `Endpoints` in src/wizard.rs, losing their kebab-case rename
- `TokenShowOutput` in src/token/show.rs
- `TokenInspectOutput` in src/token/inspect.rs
- `AuthGetOutput` in src/auth/get.rs, with both its variant payloads, `AuthorizationUri` and `DeviceAuthorization`

Config types stay kebab-case. src/config.rs is deserialized from TOML, where hyphenated keys are the family convention and no jq expression reaches them. The rule governs what the printer emits, never what the loader reads.

This costs Ortie one thing worth naming. `ConfigureOutput` was made kebab-case on purpose, so the `--json` object reads key-for-key like the TOML fragment it renders alongside. After the switch the object says `clientId` where the document it carries in `document` still says `client-id`. That is the intended split: the document is config and stays hyphenated, the envelope around it is `--json` and does not.

Provider passthrough keeps its own spelling. Any field carrying a wire name verbatim keeps its explicit `#[serde(rename = "...")]`, `@odata.nextLink` and `nextPageToken` being the family's standing examples: those are the provider's names, not names derivable from a Rust field. `rename_all` leaves an explicit `rename` alone, which is correct, and nobody should later "fix" it.

## The alias trap

`#[serde(alias = "...")]` is a deserialization-only attribute. It teaches `Deserialize` to accept a second spelling and does nothing at all to `Serialize`, so it cannot make an output type emit both `client-id` and `clientId` through a transition. An output type only serializes, so an alias on one is pure decoration. This is easy to reach for and costs real time to disprove.

The two real options are to twin the keys in the printer (emit both spellings, which doubles the payload and makes the published schema describe two names for one value) or to accept the break at the major. The decision is to accept it: a major is where a key rename is allowed to show.

## Out of scope

No CHANGELOG entry: nothing changes for a user until 3.0. The schema files `json-schema` writes change with the keys, so consumers pinned on them regenerate then.
