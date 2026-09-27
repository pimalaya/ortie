---
cairn: log
change: json-schema
landed: 2026-08-29
---

# Published a JSON Schema per data command, and stopped leaking two secrets

`ortie json-schema` prints the JSON Schema of a command's `--json` payload, or writes one file per command into `--dir`, the way `manual` does. Four commands emit data and carry a key: `ortie-configure`, `ortie-auth-get`, `ortie-token-show` and `ortie-token-inspect`. `token refresh` and `auth resume` print a confirmation, so they carry none, and the REPL carries none of its own, dispatching the same leaves and printing their payloads unchanged.

Writing the schemas is what forced the two findings the proposal predicted.

`token inspect` is documented as printing the metadata around the token rather than the token itself, and its text rendering always did. Its JSON payload was a transparent newtype over io-oauth's success params, whose custom serializers hand out `access_token` and `refresh_token` in clear, so the two renderings of one command disagreed on whether it discloses secrets. It now prints an owned `TokenInspectOutput` carrying what the text always showed: `token_type`, `issued_at`, `expires_in`, `with_refresh_token` and `scope`. The boolean replaces the `refresh_token` string rather than reusing the key with a new type, so a reader of the old field fails loudly instead of reading `false` as a token. This is breaking for a caller reading either secret from `token inspect`, and the changelog says where the raw token now comes from.

`auth get` emitted two unrelated structs under one command name. They are now the variants of an untagged `AuthGetOutput`, whose `Display` delegates, so the printed value and the described value are one type and the key maps to one schema. Untagged serialization means the emitted JSON is byte for byte what it was.

The printed types took the family's `<Command>Output` naming, reversing what this proposal's draft said. The draft argued Comodoro registers `Timer` and `GeneratedConfig` without the suffix; the cross-product scan since made the suffix a rule across the nine binaries. `AccessToken` is `TokenShowOutput`, and the wizard's `OauthConfig` is `ConfigureOutput`, which also removes a `*Config` type from outside `config.rs`, where the crate header says the configuration DTOs live. `AuthorizationUri` and `DeviceAuthorization` stay named as they are, being variant payloads rather than a command's output.

Two audits ran with it, both cheap and both worth keeping.

The emitted-versus-described check found real drift: schemars marks a field required unless it is optional or defaulted, and `skip_serializing_if` alone does not tell it anything, so the wizard's `default`, `scopes` and `extras` were described as required while the emission omits them whenever they are empty. `#[schemars(default)]` on the three fixes it, and the generated `ortie-configure` schema now requires only what always appears.

The stdout audit found nothing, which is the answer that mattered here since these payloads carry tokens. Every direct `println!` in `auth get`, including the manual-resume instructions and the browser fallbacks, sits on a path the JSON mode returned before, and the REPL's prompt goes to stderr. Its inter-command newline is the only unconditional stdout write, and it makes a `--json` session line-delimited JSON rather than broken JSON.

A test walks the clap tree and fails if a registry key names no command, taken from neverest's version of this work, so a renamed subcommand cannot leave a schema behind describing an invocation nobody can type.

Verified: build, fmt and clippy clean; 38 unit tests and 6 integration tests pass, three of them new. `json-schema` was run both ways against the real binary, and the device-grant integration test, which reads the `auth get --json` payload, passes unchanged.

Spec updated: output (new capability: A data command prints a named output type, A schema per structured-output command, The schema is the printed type, Only Ortie's own types are described, Commands carrying a schema, One command one schema, A key names a command that exists, Nothing bypasses the printer under JSON); token (ADDED: Token inspect reports metadata not secrets).
