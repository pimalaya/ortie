---
cairn: tasks
id: wizard-no-grant-test-named-entry
---

# Tasks

- [x] Delete `src/wizard/check.rs`, its module declaration and its call.
- [x] Point the welcome banner at `ortie auth get` instead of announcing an authorization check.
- [x] Prompt for the keyring entry in the storage step, defaulted to the account name.
- [x] Use the entry verbatim (no `ortie/` namespace) for both the read and the write command.
- [x] Update `src/main.rs` and `src/wizard.rs` headers, the README and the CHANGELOG.
- [x] Run `cargo fmt`, `cargo clippy` and `cargo test`.
- [x] Fold the delta into `cairn/spec/discovery.md`, write the log entry, set this change to landed.
