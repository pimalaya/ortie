---
cairn: delta
change: live-provider-tests
---

## ADDED Requirements

### Requirement: Live provider suites
The binary SHALL carry one ignored integration suite per provider (Google, Microsoft, Fastmail) under tests/, run with `--ignored`, split into an `anonymous` module that touches no account and a `credentialed` module that runs in CI only in the upstream repository. Secrets SHALL reach the binary through secret commands, never argv or config files, and no test SHALL print a token or secret.

## MODIFIED Requirements

## REMOVED Requirements
