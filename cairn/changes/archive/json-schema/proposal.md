---
cairn: change
id: json-schema
status: landed
created: 2026-08-15
---

# Publish a JSON Schema per structured-output command

Tracked by [issue #14](https://github.com/pimalaya/ortie/issues/14).

## Why

Every Ortie command takes `--json`, and nothing anywhere says what comes out of it. A script wrapping `ortie token inspect --json` learns the payload by running it once and hoping, or by reading `src/token/inspect.rs`. That is the gap Himalaya closed with `json-schema <DIR>` and Comodoro took right after: one JSON Schema file per command emitting structured output, generated from the very Rust types the commands hand to `printer.out`, so the description cannot drift from the emission.

Ortie is the product where this matters most and has it least. Its whole purpose is to be called by another program: a mail client asking for a fresh access token, a shell function feeding `--json` into `jq`, a wizard fragment consumed by an installer. Those callers are machines, and machines are exactly who a schema is for.

Writing the schemas also forces a look at what is really serialized, and two things do not survive it.

`token inspect` prints metadata by design: its help says "unlike `token show`", its text rendering says `With refresh token: true` rather than the token. Under `--json` it is a transparent newtype over io-oauth's `Oauth20AccessTokenSuccessParams`, whose custom serializers expose both `access_token` and `refresh_token` in clear. The two renderings of one command disagree on whether it hands out secrets, and publishing a schema would put that in writing.

`auth get` emits two different payloads under one command name, the authorization-code handoff and the device-authorization handoff, from two structs that no type relates. A registry maps one command to one schema, so the relation has to exist somewhere; today it exists only in the reader's head.

## What

A `json-schema <DIR>` command, taken from `pimalaya_cli::clap::commands::JsonSchemaCommand` as the other two products do, sitting beside `manuals` and `completions` as a third meta command. It writes one `<key>.json` file per entry into the given directory.

A `src/json_schema.rs` registry mapping a CLI-invocation key to the schema of the type the command prints. Keys mirror the man page names, `ortie-<cmd>-<subcmd>`, so `ortie-token-show`, `ortie-token-inspect`, `ortie-auth-get` and `ortie-configure`. Four entries, one per command that emits data.

`token inspect` stops serializing the token response and gains an owned output type carrying what its text rendering already carries: token type, issuance timestamp, remaining lifetime, whether a refresh token is held, granted scope. This is a breaking change to `--json` and takes a Changed entry in the changelog: a caller reading `.access_token` or `.refresh_token` from `token inspect` must move to `token show` or to storage.

`auth get` gains an untagged enum over its two handoff payloads, built by both branches so the printed value and the described value are one type, its `Display` delegating to the variant.

Every registered type is Ortie's own. Nothing in the schema surface comes from io-oauth, which carries no `schemars` feature, so this change needs no io-oauth release and no cross-repo coordination. The foreign types that remain as *fields* (the authorization `Url`, the CSRF state, the PKCE verifier) already serialize through `serialize_with` into strings, and are described as strings with `#[schemars(with = "String")]`.

The schema describes what is emitted, not what the struct looks like. Where `skip_serializing_if` hides a field, the schema says optional; the wizard's `default`, `scopes` and `extras` are the ones to check, since schemars marks a skipped `Vec` or map required unless told otherwise.

## Scope / non-goals

No new output. `token refresh` and `auth resume` keep printing a `Message` confirmation, so they get no key. Both do produce data (a token with an expiry), and turning them into output types is the right follow-up, but it is a behaviour change to two commands and does not belong in a change whose job is to describe what already exists.

No REPL entries. The REPL dispatches the same `token` and `auth` leaves and prints their payloads verbatim, so its lines are already described by the four keys; the registry says so in prose rather than duplicating them.

Renames, reversing what this proposal first said. The draft kept `AccessToken` and `OauthConfig` on the grounds that Comodoro registers `Timer` and `GeneratedConfig` without an `Output` suffix. The cross-product scan since then made the suffix a family rule rather than one product's habit: every data command prints a `<Command>Output` type deriving `Display`, `Serialize` and `JsonSchema`. So `AccessToken` becomes `TokenShowOutput`, the `token inspect` newtype becomes `TokenInspectOutput`, the `auth get` enum is `AuthGetOutput`, and `OauthConfig` becomes `ConfigureOutput`, which also stops a `*Config` type from living outside `config.rs` against what the crate header says about its two configuration layers.

The variant payloads `AuthorizationUri` and `DeviceAuthorization` keep their names: the command's output type is the enum, and the two describe what they carry better than a numbered suffix would.

No generated schemas in the repository and no CI step publishing them. The command generates on demand, like `manuals` and `completions`, and packaging can pick it up later.
