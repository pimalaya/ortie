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
Each stdin line SHALL be split as a shell splits it, quotes and escapes included, a word opening on `#` starting a comment to the end of the line, then parsed with the same `token` and `auth` command grammar as the one-shot CLI, subcommand prefixes inferred as there. A line left without words SHALL be skipped. A line setting a flag the session fixed when it started, `-a`, `-c` or a log flag, SHALL be refused naming that flag. A line that is not UTF-8 or whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

#### Scenario: A session flag on a line
- GIVEN a running `ortie repl` session
- WHEN `token show -a other` is entered
- THEN it fails with "Cannot set `-a` inside a session, start a new session with it instead", and the next line runs on the session's account

### Requirement: Auth runs against the session account
`auth get` and `auth resume` in the REPL SHALL run against the session's in-memory account, so a token they issue updates the cached token and is served by a following `token show` without re-reading storage.

### Requirement: Interactive versus piped output
Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as any failure. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line.

### Requirement: A line's JSON flag
A REPL line SHALL accept `--json`, printing that line's result or failure as JSON; the following lines SHALL keep the session's output format.
