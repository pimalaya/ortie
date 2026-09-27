---
cairn: spec
capability: output
status: current
---

# Output

Everything the user asked for goes to stdout, data and errors alike, distinguished only by the exit code; stderr carries the logs, the prompts and the wizard's framing. `--json` switches the data commands to machine-readable objects, and this capability is what describes those objects.

### Requirement: A data command prints a named output type
Each command emitting data SHALL print one type named after the invocation and suffixed `Output`, deriving `Display`, `Serialize` and `JsonSchema`, so the human rendering, the machine rendering and the schema are three faces of one declaration. A command reporting a confirmation SHALL print a `Message` instead, and no data SHALL ever travel as one.

### Requirement: A schema per structured-output command
Ortie SHALL provide a `json-schema` command printing the JSON Schema of a command's `--json` payload, and writing one file per command into `--dir` when given one, creating the directory when it does not exist and overwriting the schemas already in it. Each file SHALL be named after the CLI invocation it describes, the command path joined with hyphens and prefixed `ortie-`, mirroring how the man pages are named.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, which is the value serialized under `--json`, so the description cannot drift from the emission. Where a field is hidden by a serialization guard, the schema SHALL describe it as optional; where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Only Ortie's own types are described
Every registered type SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth or io-pim-discovery type, so the machine contract stays a decision of this repository and needs no release of the libraries below it.

### Requirement: Commands carrying a schema
The commands emitting data SHALL each carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-auth-get` and `ortie-configure`. The commands reporting a confirmation rather than data (`token refresh`, `auth resume`) carry none. The REPL carries none of its own: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too.

### Requirement: One command, one schema
A command emitting more than one payload shape SHALL print them through a single type describing the alternatives, so its key maps to one schema. `auth get` emits either the authorization-code handoff or the device-authorization handoff, and both are variants of that one type.

### Requirement: A key names a command that exists
Every registry key SHALL resolve to a real command in the CLI tree, checked by a test walking the parser, so renaming or removing a subcommand cannot leave a schema behind describing an invocation nobody can type.

### Requirement: Nothing bypasses the printer under JSON
Every write to stdout that is not the printer's SHALL be guarded by the JSON mode, so a `--json` run emits the payload and nothing else. The handoff instructions, the browser fallbacks and the wait notices of `auth get` are human text on the human path only.
