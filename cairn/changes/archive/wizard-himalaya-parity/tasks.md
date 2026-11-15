---
cairn: tasks
id: wizard-himalaya-parity
---

# Tasks

- [x] Bump io-pim-discovery to 0.5 (needed for `compose_all_within`).
- [x] Move the wizard out of `src/auth/discover.rs` into `src/wizard.rs` plus `src/wizard/{search,client,storage,check}.rs`; drop `AuthDiscoverCommand`.
- [x] Add the stderr welcome banner, skipped under `--json`.
- [x] Derive the account name from the input; remove the account name prompt.
- [x] Bound discovery with an 8 second deadline.
- [x] Resolve issuers (typed URL and discovered `OauthIssuer`) through RFC 8414 metadata into concrete grants; drop the `issuer` config field.
- [x] Remove the manual-entry path and stop with a pointer to the sample when nothing is discovered.
- [x] Probe the authorization server metadata once; feed `scopes_supported` into the scope options and the registration endpoint into the application step.
- [x] Run the discovered grant as the test, on stderr, writing the token through the chosen write command; skip with a note when untestable.
- [x] Print bare TOML on stdout, with the guidance moved to the banner.
- [x] Update `src/main.rs`, `src/cli.rs` and `src/repl.rs` docs to the new module and flow.
- [x] Update tests, run `cargo fmt`, `cargo clippy` and `cargo test`.
- [x] Fold the delta into `cairn/spec/discovery.md`, fix the stale `discover` mention in `cairn/spec/auth.md`, write the log entry, set this change to landed.
- [x] Update the README and the CHANGELOG.
