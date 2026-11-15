---
cairn: delta
change: aligned-text-output
---

## ADDED Requirements

### Requirement: A text record prints as aligned fields
A text rendering holding a record, `token inspect` and the `auth get` authorization and device requests, SHALL print it through pimalaya-cli's `fields::Fields`: one `Label: value` line per field, values aligned two spaces past the longest label's colon, a list such as the scopes one item per line under the first. A link or a command for the user to copy SHALL follow its sentence after a blank line, indented two spaces, as the welcome and clap's usage lines are.
