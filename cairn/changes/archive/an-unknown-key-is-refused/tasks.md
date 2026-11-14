---
cairn: tasks
change: an-unknown-key-is-refused
---

# Tasks

- [x] Tests first: an unknown key at every level (passes on the old code, each struct already denying unknown fields, and fails with `deny_unknown_fields` taken out); the `hooks` table refused naming the notes (fails on the old code, which answered ``unknown field `hooks` ``).
- [x] `AccountConfig::hooks` refused through `HooksRemovedConfig`.
- [x] config.sample.toml still loads with every variant.
- [x] Cargo.toml patches pimalaya-cli from ../cli.
- [x] CHANGELOG.
- [x] Fold into cairn/spec/config.md, log, archive.
