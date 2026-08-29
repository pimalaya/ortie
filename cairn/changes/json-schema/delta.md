---
cairn: delta
change: json-schema
---

# Delta

## ADDED Requirements

### Requirement: A schema per structured-output command
Ortie SHALL provide a `json-schema <DIR>` command writing one JSON Schema file per command emitting structured output, creating the directory when it does not exist and overwriting the schemas already in it. Each file SHALL be named after the CLI invocation it describes, the command path joined with hyphens and prefixed `ortie-`, mirroring how the man pages are named.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, which is the value serialized under `--json`, so the description cannot drift from the emission. Where a field is hidden by a serialization guard, the schema SHALL describe it as optional; where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Only Ortie's own types are described
Every registered type SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth or io-pim-discovery type, so the machine contract stays a decision of this repository and needs no release of the libraries below it.

### Requirement: Commands carrying a schema
The commands emitting data SHALL each carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-auth-get` and `ortie-configure`. The commands reporting a confirmation rather than data (`token refresh`, `auth resume`) carry none. The REPL carries none of its own: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too.

### Requirement: One command, one schema
A command emitting more than one payload shape SHALL print them through a single type describing the alternatives, so its key maps to one schema. `auth get` emits either the authorization-code handoff or the device-authorization handoff, and both are variants of that one type.

### Requirement: Token inspect reports metadata, not secrets
`token inspect` SHALL emit, in both renderings, the metadata of the stored token and not the token itself: the token type, when it was issued, what remains of its lifetime, whether a refresh token is held, and the granted scope. The access token and the refresh token SHALL NOT be serialized. The raw access token stays available from `token show`.

## MODIFIED Requirements

## REMOVED Requirements
