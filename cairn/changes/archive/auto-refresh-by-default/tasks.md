---
cairn: tasks
change: auto-refresh-by-default
---

- [x] `auto_refresh_default` (src/config.rs), the `--auto-refresh` help (src/token/show.rs)
- [x] `#auto-refresh = false` in config.sample.toml
- [x] Test: an account without the key auto-refreshes, `auto-refresh = false` turns it off; fails against the old default
- [x] CHANGELOG, token and config specs
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
