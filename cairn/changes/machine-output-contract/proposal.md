---
cairn: change
id: machine-output-contract
status: landed
created: 2026-10-27
---

# Adopt the shared machine output contract

## Why

pimalaya-cli now owns the machine contract every Pimalaya binary prints (cli `machine-output-contract`, commit `55b6693`): a schema for the failure object, schemas stamped with an `$id` naming the contract version, piped `--json` delimited by newlines, and a check that every payload is a JSON object, which is what an MCP client takes as structured output.

## What

- Pin pimalaya-cli at `55b6693` through `[patch.crates-io]` until its next release.
- src/json_schema.rs returns a `JsonSchemaRegistry` (bin `ortie`, the crate version) instead of a bare map, registers `ortie-error` as the uncoded `ErrorReport`, and tests that no payload root is anything but an object.
- The key test walking the clap tree exempts `ortie-error`, the one key naming no command.
- The REPL stops writing its own newline after a result under `--json`, since the printer now terminates every JSON value; text results keep it.

Ortie declares no error codes, so `main.rs` keeps evaluating through the uncoded `ErrorReport::eval`. Every root already is an object: `ortie-auth-get` is an untagged enum whose schema is an `anyOf` of two references to object definitions, which `non_object_roots` accepts. No payload is wrapped.
