---
cairn: delta
id: wizard-no-grant-test-named-entry
---

# Delta

## ADDED Requirements

### Requirement: Storage entry named, not assumed
The storage step SHALL run in two prompts, mirroring the shared credential picker: a strategy (the credential provider CLIs available on the running OS, then custom shell commands), then the entry the token lives under, seeded with the account name. The entry SHALL be used verbatim in both the read and the write command, so a keyring already holding a token for this account is named as it is rather than under a namespace Ortie picked.

## MODIFIED Requirements

### Requirement: Welcome banner frames the wizard
Bare `ortie` SHALL open with a welcome banner on stderr, before the first prompt, stating what Ortie is, what the wizard does, where every configuration field is documented, and that `ortie auth get` is what authorizes the account afterwards. The banner renders on stderr so it never pollutes a redirected fragment, and it is skipped under `--json`.

## REMOVED Requirements

### Requirement: Grant tested before printing
**Reason**: the transposition does not hold. Himalaya's test opens a silent connection with a credential it already has; Ortie's equivalent runs a grant, which hijacks the browser mid-wizard, cannot complete when the redirection is not locally capturable, and writes a real token into the keyring before the config it belongs to is saved. The wizard emits a config; `ortie auth get` authorizes it and reports every failure the test would have caught.
