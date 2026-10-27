---
cairn: delta
change: repl-errors-on-stdout
---

# Delta

## MODIFIED Requirements

### Requirement: Interactive versus piped output
When stdin is a TTY the session SHALL render a prompt on stderr and report errors on stderr. When piped it SHALL write one flushed result per command on stdout, so an application can drive it, and SHALL print a parse error or a command error through the printer on stdout as the failure object (`ortie-error` under `--json`), one per line, then continue the loop. Under `--json` each result and each failure SHALL be one JSON value per line.
