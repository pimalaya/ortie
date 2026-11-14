---
cairn: change
id: an-unknown-key-is-refused
status: landed
created: 2026-11-14
---

# An unknown key is refused

## Why

The owner decided on 2026-11-14, family-wide, that a configuration refuses every key it does not read, at every level, so a typo anywhere fails loading naming the key; and that a key the app used to read and removed is refused by name, pointing at the migration notes, rather than by serde's generic unknown-field error.

Every Ortie config struct already denies unknown fields, since 2.0.0: `Config` (the root, `accounts` alone), `AccountConfig`, `EndpointsConfig`, `StoragesConfig` and `StorageConfig`. None uses `#[serde(flatten)]`. `extras` is a free map of wire parameters by design, `grant`, `tls` and `pkce` are values, and `client-secret` is pimalaya-config's one-key `Secret` enum, which refuses a second key. What is missing is the named refusal: the `hooks` table, which Ortie read until 3.0.0 removed it (hooks-removed), falls to ``unknown field `hooks` ``, which does not say it was removed nor what replaces it.

## What

- `AccountConfig` gains `hooks: Option<HooksRemovedConfig>`, an uninhabited type whose `Deserialize` fails naming the removal, `storage.write.command` as the replacement and the CHANGELOG, whatever the table holds, as Calendula and Comodoro refuse their removed keys.
- Tests: an unknown key at the root, in an account, in `endpoints`, `storage`, `storage.read` and `storage.write`, and beside `client-secret.raw`; the `hooks` table in four shapes, refused naming the notes.
- CHANGELOG: the hooks removal entry says the table is refused by name.

## Decisions

- **The notes are the CHANGELOG.** Ortie has no MIGRATION.md; the message links https://github.com/pimalaya/ortie/blob/master/CHANGELOG.md, whose 3.0.0 entry names the replacement. A MIGRATION.md, if one is written for 3.0.0, takes the link.
- **Refused at deserialization.** The table fails the load wherever it sits, as an unknown key does, not only in the account `-a` selects.
- **Only `hooks`.** It is the one key a release read and a later one removed: the wizard and dynamic client registration wrote no key of their own, `client-certificate` is still read as an alias of `client-cert`, and `ORTIE_CONFIG` was an environment variable.
