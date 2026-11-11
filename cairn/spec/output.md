---
cairn: spec
capability: output
status: current
---

# Output

Everything the user asked for goes to stdout, data and errors alike, distinguished only by the exit code; stderr carries the logs and the welcome a missing configuration meets a person with. `--json` switches the data commands to machine-readable objects, and this capability is what describes those objects.

### Requirement: A data command prints a named output type
Each command printing a value SHALL print one type named after the invocation and suffixed `Output`, deriving `Display`, `Serialize` and `JsonSchema`, so the human rendering, the machine rendering and the schema are three faces of one declaration. A sentence SHALL be the `Display` of such a type, never a value of its own.

### Requirement: A schema per structured-output command
Ortie SHALL provide a `json-schema` command printing the JSON Schema of a command's `--json` payload, and writing one file per command into `--dir` when given one, creating the directory when it does not exist and overwriting the schemas already in it. Each file SHALL be named after the CLI invocation it describes, the command path joined with hyphens and prefixed `ortie-`, mirroring how the man pages are named.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, as printed (the serialize contract), so the description cannot drift from the emission: a key the type always prints is required, `null` or not. Where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Only Ortie's own types are described
Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth type, so the machine contract stays a decision of this repository and needs no release of the library below it. The failure object is the family's shared `ErrorReport` from pimalaya-cli.

### Requirement: Commands carrying a schema
Every command printing a value SHALL carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-listen` and `ortie-auth-resume`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help, and neither does the REPL: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.

### Requirement: One command, one schema
A command emitting more than one payload shape SHALL print them through a single type describing the alternatives, told apart by their keys, so its key maps to one schema. `auth get` emits the authorization-code handoff, the device-authorization handoff, or the receipt of a token its grant issued in-process, and all three are variants of that one type.

### Requirement: A stored token answers a receipt
A command storing a new access token (`token refresh`, `auth resume`, and `auth get` when its grant issues the token in-process) SHALL print `TokenIssuedOutput`, `{expiresIn, withRefreshToken}`: the token's lifetime in seconds as the server stated it, `null` when it stated none, and whether a refresh token came with it. The token itself SHALL NOT appear. Its text form SHALL read "Access token successfully issued" or "refreshed", with the expiry.

#### Scenario: A script reads when a refreshed token expires
- GIVEN an account holding a refresh token
- WHEN `ortie token refresh --json` runs
- THEN it prints `{"expiresIn": 3599, "withRefreshToken": true}` for a token the server issued for 3599 seconds with a new refresh token

### Requirement: A key names a command that exists
A test SHALL run pimalaya-cli's registry check over the command tree: every runnable command described, bare `ortie` and `ortie repl` aside, and every key but `ortie-error` naming a command, so renaming or removing a subcommand cannot leave a schema behind describing an invocation nobody can type, nor a new one print an undescribed value.

### Requirement: Nothing bypasses the printer under JSON
Every write to stdout that is not the printer's SHALL be guarded by the JSON mode, so a `--json` run emits the payload and nothing else. The handoff instructions and the wait notices of `auth get` are human text on the human path only.

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `ortie-error`. Every generated schema SHALL carry an `$id` naming the contract version, the major version of Ortie. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`. Configuration types SHALL stay kebab-case: what the loader reads is not what the printer emits.

### Requirement: Shared shapes and spellings
`token inspect` SHALL print `issuedAt` as an RFC 3339 instant in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

### Requirement: Text output ends with a newline
Every command's text rendering SHALL end with a newline, `token show` included, so the shell prompt starts on a line of its own. The REPL SHALL add none of its own: interactive, its prompt `ortie> ` follows the output on a line of its own; piped, results follow each other line after line.

### Requirement: A text record prints as aligned fields
A text rendering holding a record, `token inspect` and the `auth get` authorization and device requests, SHALL print it through pimalaya-cli's `fields::Fields`: one `Label: value` line per field, values aligned two spaces past the longest label's colon, a list such as the scopes one item per line under the first. A link or a command for the user to copy SHALL follow its sentence after a blank line, indented two spaces, as the welcome and clap's usage lines are.
