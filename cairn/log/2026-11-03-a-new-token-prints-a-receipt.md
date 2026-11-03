---
cairn: log
change: a-new-token-prints-a-receipt
date: 2026-11-03
---

# A new token prints a receipt

The deps bump moved the pimalaya-cli build dependency to 0.3 and left the main one at 0.2, which resolved to crates.io 0.2.6 once the git patch was gone: both are 0.3 now. pimalaya-cli 0.3 has no `Message`, so `TokenIssuedOutput` (src/token/issued.rs), `{expiresIn, withRefreshToken}`, replaces the two sentences: `token refresh` and `auth resume` print it, and `auth get` prints it as its `Issued` alternative after a client credentials grant or a completed interactive one. `report_token_issued` (src/auth/get.rs) persists and fires the hooks, then answers the receipt.

src/json_schema.rs builds the registry with `JsonSchemaRegistry::new` and `insert` (serialize contract), registers `ortie-token-refresh` and `ortie-auth-resume`, and `the_registry_matches_the_command_tree` asserts `check` is empty, bare `ortie` and `ortie repl` silent, in place of the hand-written walk.

Capabilities moved: output (cairn/spec/output.md: "A stored token answers a receipt" added; "A data command prints a named output type", "The schema is the printed type", "Commands carrying a schema", "One command, one schema" and "A key names a command that exists" modified). `cargo test --all-features` passed, clippy clean.
