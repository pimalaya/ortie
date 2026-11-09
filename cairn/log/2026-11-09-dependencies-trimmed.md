---
cairn: log
change: dependencies-trimmed
landed: 2026-11-09
---

# Dependencies trimmed

Every dependency in Cargo.toml now takes `default-features = false` and names its features. Most of that changes nothing in the binary: the defaults of anyhow, schemars, serde_json and shlex were features Ortie uses, and dirs, humantime, log and open had none. The one default that cost something was clap's `color`, gone with `wrap_help`, so help and usage carry no colour. The dev-dependencies env_logger, rustls and rustls-platform-verifier, which no test used, are gone; rustls's defaults had compiled aws-lc-rs into every test build. tempfile lost `getrandom`.

`idna_adapter = "=1.0.0"` in Cargo.toml runs url's IDNA without a Unicode back end, dropping the 21 ICU4X crates: a non-ASCII host is refused when its URL is parsed, and an internationalised one is written in Punycode. `a_non_ascii_host_is_refused` (src/config.rs) was seen failing on the ICU4X back end.

`auth get` opens no browser: open, is-wsl and is-docker are gone. The authorization text reads "Open the link in your browser to authorize:", and the device text sends the user to the complete verification URI to check the code when the server sends one, to the verification URI to type it otherwise.

The `notify` feature is gone with notify-rust, `NotifyConfig`, the hooks' `notify` tables, `notify_with` (src/account.rs), the direct dirs and shellexpand dependencies, the `vendored` forward and package.nix's dbus branch (`nix eval` of the package still succeeds). A `notify` table is refused as an unknown key; `a_hook_notify_table_is_refused` was seen failing on the previous code built with `--all-features`. config.sample.toml shows the hooks as commands, `notify-send` among them; its former example single-quoted `$EXPIRES_IN`, which the shell never expanded. The active hook-variables-camel-case change dropped its notification template and keeps the renames. pimalaya/.github GUIDELINES.md cli-008 now reads that a hook may show a `notify`, which a product leaves out when its events need no desktop notification.

pimalaya-cli is pinned through `[patch.crates-io]` to 65d6d30, its logs-and-help-stay-plain change: its `terminal` feature no longer turns on env_logger's `auto-color` and clap's `wrap_help`, so logs carry no colour and anstream, terminal_size, rustix and their parts leave the tree. clap wraps nothing without `wrap_help`: help prints each paragraph on one line, which the owner chose over a fixed width bringing terminal_size and rustix back.

Capabilities moved: packaging (cairn/spec/packaging.md: "TLS-only feature set" modified, "Hosts are ASCII" and "Dependencies name their features" added), config (cairn/spec/config.md: "Hooks" and "v1 config compatibility" modified), auth (cairn/spec/auth.md: "Authorization code grant" modified), output (cairn/spec/output.md: "Nothing bypasses the printer under JSON" modified).

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features` (21 unit tests and the integration suites), `cargo check --no-default-features` with each of rustls-ring, rustls-aws and native-tls, `cargo deny check`. Release binary (fat LTO, stripped): 4,520,680 bytes before, 4,354,552 with pimalaya-cli 0.3.1, 4,342,976 with it pinned (-177,704, -3.9%); 141 crates in the normal tree before, 117 then 107.

Left:

- pimalaya-cli is pinned through `[patch.crates-io]` to 65d6d30 (logs-and-help-stay-plain, pushed 2026-11-09): Ortie's next crates.io release needs a pimalaya-cli release carrying it, the patch being ignored on publish.
- io-oauth drawing random bytes from getrandom rather than rand (chacha20, rand, rand_core, a second getrandom), and pimalaya-config leaving out toml's writer: their own repositories.
- cli-007 in pimalaya/.github GUIDELINES.md still describes the wizard's first-run welcome.
