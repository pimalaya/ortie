---
cairn: log
change: welcome-without-config
landed: 2026-11-10
---

# Welcome without config

`take_account` (src/cli.rs) returns `ConfigMissingError { path }` for a missing configuration, its message now "No configuration found at <path>, start from the sample: <config.sample.toml URL>". `meet_bare_invocation` returns the same error for a bare `ortie` when `from_paths_or_default` finds nothing, and prints the help otherwise, a configuration that fails to parse included. `main` downcasts the result: with stderr a terminal and `--json` off it prints `welcome(path, config_paths)` on stderr and exits 1; anything else goes to `ErrorReport` as before.

The welcome frames Ortie in one line, says no configuration was found at the path, gives `curl -sSL --create-dirs -o '<path>' <raw config.sample.toml>` to create it, invites the reader to edit it and uncomment the options they need, then gives `invocation(config_paths, None)` followed by `auth get`, the sample's account being the default one. Each paragraph is one line, folded by the terminal as the help is. `shell_single_quote` (src/auth/get.rs) became `pub(crate)` for it. A first version appended config.samples/google.toml; the owner moved it to config.sample.toml the same day, with the providers folded into it (providers-in-the-sample), and asked for it shorter: a paragraph on what Ortie does and one walking the provider sections and the storage commands went. `a_missing_configuration_is_typed_for_the_welcome` guards the downcast `main` relies on, `the_welcome_commands_run_as_printed` the quoting of `/a b/it's.toml`.

Seen under `script` on a scratch home: the welcome for bare `ortie` and for `-c <missing> token show` (with `-c '<missing>'` in its `auth get`), exit 1; piped stderr and `--json` print the error; bare `ortie -c config.sample.toml` prints the help. The welcome's `curl` created the missing directories and downloaded the file.

Capabilities moved: config (cairn/spec/config.md: "A missing configuration meets a person with the welcome" added, "A bare invocation shows the help" modified, a missing configuration pointing at config.sample.toml), output (cairn/spec/output.md: stderr carries the welcome too).

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`.
