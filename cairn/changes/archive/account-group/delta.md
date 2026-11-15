---
cairn: delta
change: account-group
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: The list reads the configuration only
(account) `ortie account list`, alias `ls`, SHALL print every account the configuration declares, sorted by name, as pimalaya-cli's `AccountListOutput`: `{"accounts": [{"name", "default", "backends", "notBuilt"}]}`, `default` true on the account marked `default = true`, `backends` and `notBuilt` always empty, Ortie having no backend. It SHALL take no account, so it runs whatever `-a` names and however many accounts are marked default, and SHALL reach nothing: no storage command runs and no request is sent.

#### Scenario: Two accounts
- GIVEN a configuration holding `work`, marked default, and `home`
- WHEN `ortie account list --json` runs
- THEN it prints `home` then `work`, `work` alone with `default` true, both with empty `backends` and `notBuilt`, and exits 0

### Requirement: The check proves the storage reads and the endpoints answer
(account) `ortie account check` SHALL take the account as every account-bound command does and print pimalaya-cli's `AccountCheckOutput` with the rows `storage` then `endpoints`, both always checked. `storage` SHALL run `storage.read.command` once and pass when it exits successfully printing a token JSON that parses, an empty output failing coded `token-missing`; the token SHALL be neither printed nor checked for expiry, a token a REPL session holds SHALL be left as it is, and `storage.write.command` SHALL NOT run. `endpoints` SHALL first fail coded `endpoint-missing`, before any request, on the first endpoint the grant needs that the account leaves out (`endpoints.authorization` and `endpoints.token` for `authorization-code`, `endpoints.device-authorization` and `endpoints.token` for `device`, `endpoints.token` for the client credentials grants). It SHALL then send each configured endpoint among `endpoints.authorization`, `endpoints.device-authorization`, `endpoints.token` and `endpoints.revocation`, in that order, one request over a new connection opened as the grants open theirs: `HEAD <path and query> HTTP/1.1` with the headers `Host` and `Connection: close` and nothing else, no credential, client identifier, cookie or body. Any HTTP status line in answer SHALL count as answered; the first endpoint answering none, or no HTTP one, SHALL fail the row with "Cannot reach endpoints.<key> at <url>: <why>". `endpoints.redirection` SHALL be sent nothing. A failed row SHALL end the command with the partial-outcome status 3 once the report is printed.

#### Scenario: A reachable account
- GIVEN an authorization code account storing a token, its authorization, token and revocation endpoints on a server answering `405 Method Not Allowed`
- WHEN `ortie account check --json` runs
- THEN both rows pass, the server received three `HEAD` requests carrying only `Host` and `Connection: close`, in that order, and the write command did not run

#### Scenario: An empty store and a closed port
- GIVEN a client credentials account whose store holds nothing and whose token endpoint is a closed port
- WHEN `ortie account check --json` runs
- THEN it prints both rows, `storage` failed coded `token-missing` and `endpoints` failed naming `endpoints.token`, and exits 3

## MODIFIED Requirements

### Requirement: Only Ortie's own types are described
(output) Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth type, so the machine contract stays a decision of this repository and needs no release of the library below it. The family's shared shapes from pimalaya-cli are the exception: the failure object `ErrorReport` and the account outputs `AccountListOutput` and `AccountCheckOutput`.

### Requirement: Commands carrying a schema
(output) Every command printing a value SHALL carry one key: `ortie-account-list`, `ortie-account-check`, `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-listen` and `ortie-auth-resume`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help, and neither does the REPL: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.
