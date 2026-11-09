---
cairn: tasks
change: hooks-removed
---

- [x] src/config.rs: the hook types out; `a_hooks_table_is_refused` replaces `a_hook_notify_table_is_refused` (a hook command parsed before, so it fails on the old code)
- [x] src/account.rs: hook commands and runners out
- [x] src/auth/get.rs, src/auth/resume.rs, src/token/refresh.rs, src/mcp.rs: hook calls out, `device_poll_client_error_hook_params` and its two tests with them
- [x] config.sample.toml, README, CONTRIBUTING, src/main.rs header, CHANGELOG (the `notify` entry widened to the hooks)
- [x] hook-variables-camel-case abandoned
- [x] `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
