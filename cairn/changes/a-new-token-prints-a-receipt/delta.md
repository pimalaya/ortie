---
cairn: delta
change: a-new-token-prints-a-receipt
---

## ADDED Requirements

### Requirement: A stored token answers a receipt
A command storing a new access token (`token refresh`, `auth resume`, and `auth get` when its grant issues the token in-process) SHALL print `TokenIssuedOutput`, `{expiresIn, withRefreshToken}`: the token's lifetime in seconds as the server stated it, `null` when it stated none, and whether a refresh token came with it. The token itself SHALL NOT appear. Its text form SHALL read "Access token successfully issued" or "refreshed", with the expiry.

#### Scenario: A script reads when a refreshed token expires
- GIVEN an account holding a refresh token
- WHEN `ortie token refresh --json` runs
- THEN it prints `{"expiresIn": 3599, "withRefreshToken": true}` for a token the server issued for 3599 seconds with a new refresh token

## MODIFIED Requirements

### Requirement: A data command prints a named output type
Each command printing a value SHALL print one type named after the invocation and suffixed `Output`, deriving `Display`, `Serialize` and `JsonSchema`, so the human rendering, the machine rendering and the schema are three faces of one declaration. A sentence SHALL be the `Display` of such a type, never a value of its own.

### Requirement: The schema is the printed type
Each schema SHALL be derived from the Rust type the command hands to the printer, as printed (the serialize contract), so the description cannot drift from the emission: a key the type always prints is required, `null` or not. Where a field serializes through a custom serializer, the schema SHALL describe the shape that serializer produces.

### Requirement: Commands carrying a schema
Every command printing a value SHALL carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-auth-get`, `ortie-auth-resume` and `ortie-configure`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help or running the wizard `configure` describes, and neither does the REPL: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.

### Requirement: One command, one schema
A command emitting more than one payload shape SHALL print them through a single type describing the alternatives, told apart by their keys, so its key maps to one schema. `auth get` emits the authorization-code handoff, the device-authorization handoff, or the receipt of a token its grant issued in-process, and all three are variants of that one type.

### Requirement: A key names a command that exists
A test SHALL run pimalaya-cli's registry check over the command tree: every runnable command described, bare `ortie` and `ortie repl` aside, and every key but `ortie-error` naming a command, so renaming or removing a subcommand cannot leave a schema behind describing an invocation nobody can type, nor a new one print an undescribed value.
