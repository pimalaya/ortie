---
cairn: tasks
id: wizard-no-custom-prompts
---

# Tasks

- [x] Drop the `custom` prompts; the entry returns the scope source directly and leaves the client id empty.
- [x] Skip the application step when the custom entry is the only candidate.
- [x] Label the entry as filled in by hand.
- [x] Explain the empty client id on stderr next to the printed fragment, and withhold the `auth get` hint until an application is there.
- [x] Update the client, scope and wizard module headers.
- [x] Run `cargo fmt`, `cargo clippy` and `cargo test`.
- [x] Fold the delta into `cairn/spec/discovery.md`, write the log entry, set this change to landed.
- [x] Update the README and the CHANGELOG.
