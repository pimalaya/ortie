---
cairn: delta
change: text-output-ends-with-newline
---

## ADDED Requirements

### Requirement: Text output ends with a newline
Every command's text rendering SHALL end with a newline, `token show` included, so the shell prompt starts on a line of its own. The REPL SHALL add none of its own: interactive, its prompt opens with a blank line; piped, results follow each other line after line.

## MODIFIED Requirements

### Requirement: Show the raw access token
`token show` SHALL read the token from storage and print the raw access token on stdout, followed by a newline like every text output, suitable for piping: `$(ortie token show)` and secret readers strip it. Under `--json` it prints the token as a JSON object.
