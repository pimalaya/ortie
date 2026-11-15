---
cairn: delta
change: repl-and-mcp-polish
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Interactive versus piped output
Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, its text form hinting `Restart the session with --log-level to enable more verbose logs` while debug logs are off, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as any failure. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line. The help's usage line SHALL name `ortie` before the command, as on the command line.

#### Scenario: A command's help in a session
- GIVEN a running `ortie repl` session
- WHEN `help token show` is entered
- THEN the help's usage line reads `Usage: ortie token show [OPTIONS]`

## REMOVED Requirements
