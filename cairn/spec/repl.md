---
cairn: spec
capability: repl
status: current
---

# REPL

`ortie repl` is a persistent session bound to one account. It resolves the account token once and answers `token` commands over stdin/stdout, so a keyring that confirms disclosure per process is unlocked a single time for the whole session instead of on every command. The persistence is private to the process that spawned it (the stdio pipe is the authorization boundary), so it does not hand the token to other processes the way a shared agent would.

### Requirement: Persistent session
`ortie repl` SHALL start a persistent session bound to one account (the `--account` or default). It SHALL read one command per line from stdin, run it against an in-memory account, write the result to stdout, and loop until EOF or a quit command. The account's stored token SHALL be resolved from storage at most once for the session (memoized on first use) and reused, so the read storage command runs once. On exit the in-memory secret SHALL be zeroized.

#### Scenario: repeated show without re-reading storage
- GIVEN a running `ortie repl` session whose token was resolved on first use
- WHEN `token show` is entered several times
- THEN each prints the access token without re-running the read storage command

### Requirement: Command grammar reuse
Each stdin line SHALL be parsed with the same `token` command grammar as the one-shot CLI. A parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

### Requirement: Interactive versus piped output
When stdin is a TTY the session SHALL render a prompt on stderr; when piped it SHALL write results on stdout and errors on stderr, one flushed result per command, so an application can drive it.
