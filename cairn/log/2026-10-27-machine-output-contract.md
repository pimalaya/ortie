---
cairn: log
change: machine-output-contract
landed: 2026-10-27
---

# Adopted the shared machine output contract

pimalaya-cli is pinned at `55b6693` through `[patch.crates-io]` until its next release. src/json_schema.rs now returns a `JsonSchemaRegistry` (bin `ortie`, the crate version), so `json-schema` stamps each schema with `$id` = `https://pimalaya.org/schemas/ortie/v2/<key>.json`, and registers `ortie-error`, the uncoded `ErrorReport` any command prints on failure. Ortie declares no error codes, so the object carries no `code` and main.rs keeps `ErrorReport::eval`.

A test asserts every payload root is an object. All already were: `ortie-auth-get`, an untagged enum, is an `anyOf` of two `$ref`s to object definitions, which `non_object_roots` accepts. The test walking the clap tree for each key skips `ortie-error`, the one key naming no command.

The printer now ends each JSON value with a newline, flushed when piped. The REPL used to write its own newline after every successful command, which under `--json` would have doubled into empty lines between objects; it now writes it only for a text result, which carries no trailing newline of its own.

Verified: fmt and clippy (all features, all targets) clean; 40 unit and 7 integration tests pass with all features, 17 unit tests without the wizard (`--no-default-features --features rustls-ring`). A plain `--no-default-features` build fails in io-oauth's `Oauth20ClientStd::connect` for lack of a TLS provider, as it did before.

Spec updated: output (ADDED: The JSON contract is described and versioned; MODIFIED: Only Ortie's own types are described, Commands carrying a schema, A key names a command that exists).
