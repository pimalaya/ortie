---
cairn: spec
capability: repl
status: current
---

# REPL

`ortie repl` is a persistent session bound to one account. It resolves the account token once and answers the commands of the binary over stdin/stdout, so a keyring that confirms disclosure per process is unlocked a single time for the whole session instead of on every command. The persistence is private to the process that spawned it (the stdio pipe is the authorization boundary), so it does not hand the token to other processes the way a shared agent would.

### Requirement: Persistent session
`ortie repl` SHALL start a persistent session bound to one account (the `--account` or default). It SHALL read one command per line from stdin, run it against an in-memory account, write the result to stdout, and loop until EOF or a quit command. The account's stored token SHALL be resolved from storage at most once for the session (memoized on first use) and reused, so the read storage command runs once. On exit the in-memory secret SHALL be zeroized.

#### Scenario: repeated show without re-reading storage
- GIVEN a running `ortie repl` session whose token was resolved on first use
- WHEN `token show` is entered several times
- THEN each prints the access token without re-running the read storage command

### Requirement: Command grammar reuse
Each stdin line SHALL be split as a shell splits it, quotes and escapes included, a word opening on `#` starting a comment to the end of the line, then parsed with the binary's whole command grammar, without the binary name, subcommand prefixes inferred as on the command line, and run as the command line runs it over the session's account. A line left without words SHALL be skipped. A line setting a flag the session fixed when it started, `-a`, `-c` or a log flag, SHALL be refused naming that flag. `account check` SHALL check the session's account, running its read storage command again and leaving the token the session holds as it is. A line that is not UTF-8 or whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

#### Scenario: A session flag on a line
- GIVEN a running `ortie repl` session
- WHEN `token show -a other` is entered
- THEN it fails with "Cannot set `-a` inside a session, start a new session with it instead", and the next line runs on the session's account

#### Scenario: An account line
- GIVEN a running `ortie repl --json` session
- WHEN `account list` then `token show` are entered
- THEN the first prints the accounts of the configuration and the second the session's token

### Requirement: Auth runs against the session account
`auth get` and `auth resume` in the REPL SHALL run against the session's in-memory account, so a token they issue updates the cached token and is served by a following `token show` without re-reading storage.

### Requirement: Interactive versus piped output
Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, its text form hinting `Restart the session with --log-level to enable more verbose logs` while debug logs are off, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as the failure object coded `usage-invalid`, its message without clap's `error: ` prefix. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line. The version SHALL be the one the command line prints, `ortie <version>` for `-V`. The help's usage line SHALL name `ortie` before the command, as on the command line.

#### Scenario: A command's help in a session
- GIVEN a running `ortie repl` session
- WHEN `help token show` is entered
- THEN the help's usage line reads `Usage: ortie token show [OPTIONS]`

### Requirement: A line's JSON flag
A REPL line SHALL accept `--json`, printing that line's result or failure as JSON; the following lines SHALL keep the session's output format.

### Requirement: A session never nests
A `repl` or `mcp` line, by its name, an alias or a prefix, SHALL be refused by pimalaya-cli with `Cannot start a session inside a session`, coded `usage-invalid`, before Ortie runs anything, and the loop SHALL continue.

#### Scenario: An MCP server asked for in a session
- GIVEN a piped `ortie repl --json` session
- WHEN the line `mcp --allow-write` is read
- THEN stdout holds the failure object coded `usage-invalid` with the error `Cannot start a session inside a session`, and the next line runs
