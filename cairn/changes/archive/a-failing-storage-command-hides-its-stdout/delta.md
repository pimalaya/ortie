---
cairn: delta
change: a-failing-storage-command-hides-its-stdout
---

## MODIFIED Requirements
### Requirement: Storage commands
A storage command exiting non-zero SHALL be reported by its trimmed stderr, or by its program and exit status when it wrote none, never by its stdout, which holds the token or echoes it: in the failure object, the `storage` row of `account check` and the MCP tools alike. A failing read command's report SHALL end with ", run `<command>` yourself to see why", the command written as configured.
