---
cairn: delta
change: machine-output-contract
---

# Delta

## ADDED Requirements

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `ortie-error`. Every generated schema SHALL carry an `$id` naming the contract version, the major version of Ortie. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

## MODIFIED Requirements

### Requirement: Only Ortie's own types are described
Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth or io-pim-discovery type, so the machine contract stays a decision of this repository and needs no release of the libraries below it. The failure object is the family's shared `ErrorReport` from pimalaya-cli.

### Requirement: Commands carrying a schema
The commands emitting data SHALL each carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-auth-get` and `ortie-configure`. The commands reporting a confirmation rather than data (`token refresh`, `auth resume`) carry none. The REPL carries none of its own: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.

### Requirement: A key names a command that exists
Every registry key but `ortie-error` SHALL resolve to a real command in the CLI tree, checked by a test walking the parser, so renaming or removing a subcommand cannot leave a schema behind describing an invocation nobody can type.
