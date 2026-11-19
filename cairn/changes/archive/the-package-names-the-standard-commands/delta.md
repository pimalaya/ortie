---
cairn: delta
change: the-package-names-the-standard-commands
---

## ADDED Requirements
### Requirement: The Nix package installs what the binary generates
package.nix SHALL generate the manual pages, the shell completions and the JSON Schemas through the standard commands under their official names (`manual`, `completion`, `json-schema`), never a hidden alias, into share/man, share/completions and share/schemas, beside share/doc/ortie/config.sample.toml. Its `meta.changelog` SHALL link the GitHub release of the tag it builds (`releases/tag/<tag>`).
