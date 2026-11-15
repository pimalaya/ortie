---
cairn: delta
change: repl-takes-the-whole-grammar
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A session never nests
(repl) A `repl` or `mcp` line, by its name, an alias or a prefix, SHALL be refused by pimalaya-cli with `Cannot start a session inside a session`, coded `usage-invalid`, before Ortie runs anything, and the loop SHALL continue.

#### Scenario: An MCP server asked for in a session
- GIVEN a piped `ortie repl --json` session
- WHEN the line `mcp --allow-write` is read
- THEN stdout holds the failure object coded `usage-invalid` with the error `Cannot start a session inside a session`, and the next line runs

## MODIFIED Requirements

### Requirement: Command grammar reuse
(repl) Each stdin line SHALL be split as a shell splits it, quotes and escapes included, a word opening on `#` starting a comment to the end of the line, then parsed with the binary's whole command grammar, without the binary name, subcommand prefixes inferred as on the command line, and run as the command line runs it over the session's account. A line left without words SHALL be skipped. A line setting a flag the session fixed when it started, `-a`, `-c` or a log flag, SHALL be refused naming that flag. `account check` SHALL check the session's account, running its read storage command again and leaving the token the session holds as it is. A line that is not UTF-8 or whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

#### Scenario: A session flag on a line
- GIVEN a running `ortie repl` session
- WHEN `token show -a other` is entered
- THEN it fails with "Cannot set `-a` inside a session, start a new session with it instead", and the next line runs on the session's account

#### Scenario: An account line
- GIVEN a running `ortie repl --json` session
- WHEN `account list` then `token show` are entered
- THEN the first prints the accounts of the configuration and the second the session's token

### Requirement: Interactive versus piped output
(repl) Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, its text form hinting `Restart the session with --log-level to enable more verbose logs` while debug logs are off, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as the failure object coded `usage-invalid`, its message without clap's `error: ` prefix. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line. The version SHALL be the one the command line prints, `ortie <version>` for `-V`. The help's usage line SHALL name `ortie` before the command, as on the command line.

#### Scenario: A command's help in a session
- GIVEN a running `ortie repl` session
- WHEN `help token show` is entered
- THEN the help's usage line reads `Usage: ortie token show [OPTIONS]`

### Requirement: Commands carrying a schema
(output) Every command printing a value SHALL carry one key: `ortie-account-list`, `ortie-account-check`, `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-listen` and `ortie-auth-resume`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help, and neither does the REPL: it dispatches the commands of the binary and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.
