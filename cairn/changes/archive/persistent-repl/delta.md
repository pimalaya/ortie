---
cairn: delta
change: persistent-repl
---

## ADDED Requirements

### Requirement: Persistent session
`ortie repl` SHALL start a persistent session bound to one account (the `--account` or default). It SHALL read one command per line from stdin, run it against an in-memory account, write the result to stdout, and loop until EOF or a quit command. The account's stored token SHALL be resolved from storage at most once for the session (memoized on first use) and reused, so the read storage command runs once. On exit the in-memory secret SHALL be zeroized.

#### Scenario: repeated show without re-reading storage
- GIVEN a running `ortie repl` session whose token was resolved on first use
- WHEN `token show` is entered several times
- THEN each prints the access token without re-running the read storage command

### Requirement: Command grammar reuse
Each stdin line SHALL be parsed with the same `token` command grammar as the one-shot CLI. A parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

### Requirement: Interactive versus piped output
When stdin is a TTY the session SHALL render a prompt on stderr; when piped it SHALL use a plain line protocol with results on stdout and errors on stderr, so an application can drive it.

### Requirement: Local issuance timestamp
On every successful issuance or refresh, Ortie SHALL stamp `issued_at` with its own wall clock at receipt on the token it persists and caches, overriding any server `Date` header, so expiry is computed against a single clock. On refresh the new token SHALL be persisted to storage before the in-memory token is replaced.

## MODIFIED Requirements

### Requirement: Expiry with skew
`token show` SHALL treat a token as expired when `issued_at + expires_in` is within a fixed skew (60 seconds) of the wall clock. When `expires_in` is absent it SHALL default to one hour (3600 seconds). When `issued_at` is absent the token is assumed still valid.
