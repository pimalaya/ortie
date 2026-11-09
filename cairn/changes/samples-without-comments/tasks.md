---
cairn: tasks
change: samples-without-comments
---

- [x] config.sample.toml rewritten as one flat `[accounts.example]` table, every value listed, the default active, `exit 127` commands, one tab-aligned sentence per line (from the owner's start on `default` and `grant`)
- [x] samples/ renamed config.samples/, holding google.toml, microsoft.toml and fastmail.toml, `[accounts.<provider>]` each, the own application and other scopes as commented code (the Google secret copied from the old file by perl)
- [x] `check_sample` (src/config.rs): one `[accounts.<name>]` header, parsed as it stands and once per `variants` entry; seen failing on a misspelled commented `client-id` in microsoft.toml and a `#pkce = "plane"` in config.sample.toml
- [x] README Configuration section, CHANGELOG, src/main.rs header
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
