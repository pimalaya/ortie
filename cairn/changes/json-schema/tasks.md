---
cairn: tasks
change: json-schema
---

# Tasks

- [ ] Add the `schemars` dependency (version 1, `derive`) and move `serde_json` from dev-dependencies to dependencies, since the registry builds `Value`s.
- [ ] Add the `JsonSchema(JsonSchemaCommand)` variant to the root command tree, beside `Manuals` and `Completions`, dispatched to `json_schema::schemas()`.
- [ ] Add `src/json_schema.rs`: module header explaining the key convention (`ortie-<cmd>-<subcmd>`, mirroring the man pages), why the confirmation commands carry no schema, and why the REPL needs no keys of its own.
- [ ] Register `ortie-token-show` on `AccessToken`, deriving `JsonSchema`.
- [ ] Replace `token inspect`'s transparent newtype over `Oauth20AccessTokenSuccessParams` with an owned output carrying what the text rendering carries (token type, issued at, expiry, whether a refresh token is held, scope), so `--json` stops handing out the access and refresh tokens; register it as `ortie-token-inspect`.
- [ ] Wrap `auth get`'s two handoff payloads in an untagged enum built by both branches, its `Display` delegating to the variant; register it as `ortie-auth-get`.
- [ ] Describe the `serialize_with` fields as what they serialize to: `#[schemars(with = "String")]` on the authorization URI, the CSRF state and the PKCE code verifier.
- [ ] Register `ortie-configure` on `OauthConfig`, deriving `JsonSchema` down through `RawSecret`, `Endpoints`, `Storage`, `StorageEntry` and `StorageCommand`.
- [ ] Audit the emitted-versus-described mismatch: every field hidden by `skip_serializing_if` (the wizard's `default`, `scopes`, `extras`, and the optional endpoints) is optional in the schema.
- [ ] Tests: every registered schema builds, and the `token inspect` output serializes neither the access token nor the refresh token.
- [ ] Build/test/fmt/clippy.
- [ ] Changelog: Added for the `json-schema` command, Changed for the `token inspect` payload, naming `token show` as where the raw token now comes from.
- [ ] Fold into cairn/spec/output.md (new capability) and cairn/spec/token.md; log; land; close issue #14.
