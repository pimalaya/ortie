---
cairn: tasks
id: wizard-append-and-argv
---

# Tasks

- [x] Split `StorageCommand` into `Argv` and `Shell`; emit keyring reads as an argv.
- [x] Render TOML values through escaping helpers (basic strings, literal shell lines).
- [x] Add the save prompt, worded so each answer names its arm, defaulting to yes.
- [x] Append to an existing config file instead of asking to overwrite; create a missing file and its parent.
- [x] Keep a redirected stdout and `--json` printing without prompting.
- [x] Cover the fragment with a round-trip test through the real config loader.
- [x] Update the banner, `src/main.rs` and `src/wizard.rs` headers, the README and the CHANGELOG.
- [x] Run `cargo fmt`, `cargo clippy` and `cargo test`.
- [x] Fold the delta into `cairn/spec/discovery.md`, write the log entry, set this change to landed.
- [ ] Once validated here, port the save prompt and the append behaviour back to the Himalaya wizard.
