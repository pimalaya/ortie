---
cairn: log
change: source-tree-refresh
landed: 2026-07-13
---

# Source tree and docs refresh

Inlined the `cli/` folder into flat modules: `src/cli.rs` (root parser), `src/config.rs`, `src/account.rs`, and the command trees `src/auth.rs` + `src/auth/{discover,get,resume}.rs` and `src/token.rs` + `src/token/{show,inspect,refresh}.rs`. `src/main.rs` now carries the architecture document in its header, the way `lib.rs` does for io-oauth. Every pub item carries a doc comment whose first paragraph (two lines at most) clap renders as the `-h` summary and the rest as `--help`. CONTRIBUTING was rewritten for the pure-CLI reality, the README intro became a paragraph aligned with the Cargo description, and the CHANGELOG `[Unreleased]` was compacted from a history log into a net diff against 1.1.0.

No spec capability moved (internal reorganisation and documentation only).
