---
cairn: delta
id: wizard-storage-installed-first
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Storage entry named, not assumed
The storage step SHALL run in two prompts, mirroring the shared credential picker: a strategy (the credential provider CLIs relevant on the running OS, then custom shell commands), then the entry the token lives under, seeded with the account name. The strategies SHALL be ordered by whether their CLI is found on the `PATH`, the installed ones leading, and the ones that are not SHALL say so rather than be hidden: a missing provider is one package install away, and the configuration written for it is correct either way. The order within each group SHALL be the picker's own, most native first. The entry SHALL be used verbatim in both the read and the write command, so a keyring already holding a token for this account is named as it is rather than under a namespace Ortie picked.

## REMOVED Requirements

None.
