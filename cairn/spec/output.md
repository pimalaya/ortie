---
cairn: spec
capability: output
status: current
---

# Output

Everything the user asked for goes to stdout, data and errors alike, distinguished only by the exit code; stderr carries the logs, the prompts and the wizard's framing. `--json` switches the data commands to machine-readable objects, and this capability is what describes those objects.

### Requirement: A data command prints a named output type
Each command printing a value SHALL print one type named after the invocation and suffixed `Output`, deriving `Display`, `Serialize` and `JsonSchema`, so the human rendering, the machine rendering and the schema are three faces of one declaration. A sentence SHALL be the `Display` of such a type, never a value of its own.

### Requirement: A schema per structured-output command
Ortie SHALL provide a `json-schema` command printing the JSON Schema of a command's `--json` payload, and writing one file per command into `--dir` when given one, creating the directory when it does not exist and overwriting the schemas already in it. Each file SHALL be named after the CLI invocation it describes, the command path joined with hyphens and prefixed `ortie-`, mirroring how the man pages are named.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, as printed (the serialize contract), so the description cannot drift from the emission: a key the type always prints is required, `null` or not. Where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Only Ortie's own types are described
Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth or io-pim-discovery type, so the machine contract stays a decision of this repository and needs no release of the libraries below it. The failure object is the family's shared `ErrorReport` from pimalaya-cli.

### Requirement: Commands carrying a schema
Every command printing a value SHALL carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-resume` and `ortie-configure`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help or running the wizard `configure` describes, and neither does the REPL: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.

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
Every write to stdout that is not the printer's SHALL be guarded by the JSON mode, so a `--json` run emits the payload and nothing else. The handoff instructions, the browser fallbacks and the wait notices of `auth get` are human text on the human path only.

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `ortie-error`. Every generated schema SHALL carry an `$id` naming the contract version, the major version of Ortie. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`. Configuration types SHALL stay kebab-case, including the TOML document `configure` prints: what the loader reads is not what the printer emits.

### Requirement: Shared shapes and spellings
`configure` SHALL print `{name, default, document}`, the account name, whether it claims the default, and the TOML fragment its text form prints, every key present; `document` is `null` when `--name` kept an account the configuration already held, the text form then naming the kept account. `token inspect` SHALL print `issuedAt` as an RFC 3339 instant in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.
