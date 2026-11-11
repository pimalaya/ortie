---
cairn: tasks
change: samples-replace-the-wizard
---

# Tasks

## Samples first, while the wizard still holds the data

- [x] samples/*.toml as the proposal's table lists, the client ids, endpoints, redirections and scopes taken from `KNOWN_APPS` (src/wizard/client.rs) and the README provider blocks; the Google secret read from `pass` into a variable, never printed
- [x] Check whether calendula reaches Google Tasks and Graph To Do: it does not (its todos run on CalDAV, vdir and pimdir), so the calendar samples carry no Tasks scope
- [x] samples/fastmail-own-application.toml: the RFC 7591 registration request in a comment, shaped after the 2026-07-15 dynamic-client-registration log (private-use redirection only), the endpoints from Fastmail's live metadata; not run, a registration being a write at Fastmail
- [x] A test parsing every file of samples/ through `Config::from_paths`, one account each; seen failing on a deliberately broken sample (a misspelled `endpoints.redirections`), naming the file

## Removal

- [x] Delete src/wizard.rs and src/wizard/ (client, scope, search, storage)
- [x] src/cli.rs: drop the `configure` variant (alias `wizard`) and both `offer_configuration`, `take_account` losing its printer; src/main.rs: drop the bare-invocation offer, a bare `ortie` showing the help
- [x] The missing-configuration failure points at the samples, the cfg-split `NO_CONFIG_HINT` gone
- [x] src/json_schema.rs: drop `ortie-configure`
- [x] Cargo.toml: drop the `wizard` feature from `default` and the list, io-pim-discovery and its forwards in `native-tls`, `rustls-aws`, `rustls-ring` and `vendored`; pimalaya-cli keeps `terminal` only
- [x] `cargo tree -i` on crossterm, inquire, ctrlc, io-pim-discovery and comfy-table comes back empty
- [x] deny.toml: drop the `Zlib` allowance, foldhash having left with the wizard
- [x] src/main.rs header and CONTRIBUTING.md reworded without the wizard

## Docs

- [x] README: drop the "Configuration wizard" and "Dynamic client registration" features, the `wizard` feature note, and the RFC 7591, 8414 and 9728 rows of Coverage; the Configuration section becomes the samples table; Usage starts from a sample then `ortie auth get`; the redirect bullet names the port-less loopback
- [x] config.sample.toml header and `resources` comment: drop the wizard, point at samples/
- [x] CHANGELOG Unreleased: the wizard bullets that never shipped are dropped rather than added then removed (the `wizard` feature, the `configure` positional, `--service`, `--name`, Pimalaya's applications in the wizard, `configure --json`, the wizard's RFC 9728, grant and PKCE changes, the io-pim-discovery bump); Removed, BREAKING: `configure` and its `wizard` alias with the first-run offer, dynamic client registration; Added: samples/
- [x] Spec: delta folded (config, output, repl, provider-quirks), preambles reworded (auth, output, packaging, provider-quirks), discovery.md deleted
- [x] live-provider-tests (active): drop `wizard_discovers_fastmail` from its proposal and tasks, and name the samples as candidate fixtures

## Checks and landing

- [x] fmt, clippy `-D warnings`, `cargo test --all-features`, `cargo check --no-default-features --features rustls-ring`, `cargo deny check`
- [x] Release binary size and crate count before and after, recorded in the log
- [x] Delta folded, log written, status landed

## The owner's

- [x] ~~On push, allow the `GOCSPX-` value of samples/google-pimalaya-*.toml if push protection stops it~~ Done on config.sample.toml, allowed on push; its Google client is left empty since 2026-11-11.
- [x] ~~Live: `ortie auth get` from samples/google-pimalaya-mail.toml and samples/microsoft-pimalaya-outlook.toml on the test accounts~~ Done from the sections of config.sample.toml, samples/ being gone.
- [ ] Order the releases: Ortie without `configure` ships no earlier than the CLI releases that stop calling it
