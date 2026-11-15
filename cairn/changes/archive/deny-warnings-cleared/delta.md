---
cairn: delta
change: deny-warnings-cleared
---

## ADDED Requirements

### Requirement: Dependency checks pass clean
`cargo deny check` SHALL pass with no warning over the targets deny.toml lists, which are the release targets and the ones people build from source. A duplicate the tree cannot avoid SHALL be skipped by name, with its reason.

## MODIFIED Requirements

## REMOVED Requirements
