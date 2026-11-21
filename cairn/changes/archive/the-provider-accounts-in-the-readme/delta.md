---
cairn: delta
id: the-provider-accounts-in-the-readme
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: One sample configuration, the provider accounts in the README (config)
Renamed from "One sample configuration, with a section per provider". config.sample.toml ends with a comment pointing to the README, then the custom section alone. The README carries one subsection per provider, each with one toml block per account (`google`, `microsoft-graph`, `microsoft-imap-smtp`, `fastmail`) holding the keys the sample sections held and `pass` storage commands. The test suite parses the sample with the custom section, and each README block through the same loader, checking the account it names.

### Requirement: Live provider suites (packaging)
The `anonymous` accounts come from the README's provider accounts through tests/common.rs `readme_account`, so a provider drifting from the README fails them.

### Provider quirks intro, Current Google endpoints, Fastmail resource and redirect (provider-quirks)
The README's provider accounts apply the quirks, leave the Google client empty and carry the client `df6e41ae`, where the sample sections did.

## REMOVED Requirements

None.
