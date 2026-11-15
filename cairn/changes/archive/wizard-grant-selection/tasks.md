---
cairn: tasks
id: wizard-grant-selection
---

# Tasks

- [x] Compare grants by flow and issuer host in `same_grant`, so endpoint spellings of one authorization server fold into one entry.
- [x] Drop the token endpoint from the grant pick-list label, and omit the service list when there is none.
- [x] Carry a single discovered grant through without a pick list.
- [x] Turn `grant_of` into `grants_of`, returning every grant the metadata advertises; update the issuer resolution and the typed-issuer search.
- [x] Shorten `DISCOVERY_TIMEOUT` to 6 seconds.
- [x] Add `src/wizard/scope.rs` holding the scope step and the source the application step hands it.
- [x] Run the application step before the scope step; have it return the scope source, and prompt the scopes from inside the dynamic registration branch before registering.
- [x] Reword the custom client id prompt and always serialize the client id key.
- [x] Update tests, run `cargo fmt`, `cargo clippy` and `cargo test`.
- [x] Fold the delta into `cairn/spec/discovery.md`, write the log entry, set this change to landed.
- [x] Update the CHANGELOG.
