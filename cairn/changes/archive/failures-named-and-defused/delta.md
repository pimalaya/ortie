---
cairn: delta
change: failures-named-and-defused
---

## ADDED Requirements
### Requirement: A log file that cannot be opened fails the command
A `--log-file` Ortie cannot open or create SHALL fail the command before it runs, as any other failure: the failure object on stdout, naming the path, and exit 1, never a panic.

## MODIFIED Requirements
### Requirement: A text record prints as aligned fields
A value a server wrote, the token type, a scope, the user code and the verification URIs, SHALL be defused by pimalaya-cli's `sanitize::text` first.

### Requirement: Refresh rotation ordering
A refresh that cannot reach the token endpoint SHALL fail as "Refresh access token error", the connection error as its cause.
