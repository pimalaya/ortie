---
cairn: log
change: config-flag-repeated
landed: 2026-11-10
---

# Config flag repeated

`ConfigPathsArg` (src/cli.rs) lost its `env = "ORTIE_CONFIG"` and its `value_delimiter = ':'`: `-c` takes one path per occurrence and is repeated to merge several files, the first the base. `a_config_path_is_never_split` parses `-c C:\a.toml -c b:c;d.toml` into those two paths; with the delimiter back it got `["C", "\\a.toml", "b", "c;d.toml"]`. clap's `env` feature left Cargo.toml, Ortie having been its only user in the build; no crate left. `invocation` (src/auth/get.rs) prints one single-quoted `-c` per path in the `auth listen` and `auth resume` commands.

The README Configuration section and the `--config` help show the repeated flag (`-c public.toml -c private.toml`), and the CHANGELOG lists both breaking changes, `-c "$ORTIE_CONFIG"` given as the way a shell variable still reaches the flag. In pimalaya/.github, GUIDELINES.md gains cli-009 (the configuration path overridden by a repeatable, never-split `-c` only, no `<NAME>_CONFIG`) and the readme-010 template drops the variable and the `:` list. pimalaya-cli's shared `-c` flag and the sibling CLIs still split on `:` and read `<NAME>_CONFIG`.

Capabilities moved: config (cairn/spec/config.md: "The configuration path is read from the environment" replaced by "The configuration path comes from `-c` only"), auth (cairn/spec/auth.md: "Manual resume fallback" carries one `-c` per path).

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`.
