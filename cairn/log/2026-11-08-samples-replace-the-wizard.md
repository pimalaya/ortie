---
cairn: log
change: samples-replace-the-wizard
landed: 2026-11-08
---

# Samples replace the wizard

The wizard is gone: src/wizard.rs and src/wizard/ (client, scope, search, storage), the `configure` command and its `wizard` alias, the offer a bare `ortie` or a command missing its configuration made, the welcome, the `wizard` cargo feature, the `ortie-configure` schema key, and dynamic client registration (RFC 7591), which only the wizard ran. A bare `ortie` shows the help; `take_account` (src/cli.rs) no longer takes the printer, and a missing configuration fails naming the path it looked for and pointing at https://github.com/pimalaya/ortie/tree/master/samples, the cfg-split `NO_CONFIG_HINT` removed. io-pim-discovery left Cargo.toml with its feature forwards, and pimalaya-cli keeps its `terminal` feature only, so inquire, crossterm, ctrlc and comfy-table are out of the tree; deny.toml drops the `Zlib` allowance foldhash needed.

samples/ holds ten accounts, one per use case: Pimalaya's Google Desktop client for mail (`https://mail.google.com/`), calendars (`calendar.events`, `calendar.calendarlist.readonly`) and contacts (`contacts`); Pimalaya's Entra application for Outlook IMAP and SMTP and for Graph mail, calendars and contacts, pinned to `http://localhost`; and an application of one's own for Google (CalDAV, CardDAV), Microsoft and Fastmail, the last with its registration request in a comment. The calendar samples carry no Tasks scope, calendula's Google and Graph backends handling no todos. The Google secret went in from `pass`, never printed. `every_sample_parses_into_one_account` (src/config.rs) parses each through `Config::from_paths` and was seen failing on a misspelled field. The README's Configuration section is the samples table; the provider blocks, Thunderbird's and Microsoft Graph PowerShell's clients among them, are gone.

Capabilities moved: config (cairn/spec/config.md: "A bare invocation shows the help" and "Samples, one per use case" added, "Account resolution failures name what is missing" modified, the seven wizard requirements removed), output (cairn/spec/output.md: "Commands carrying a schema", now listing `ortie-auth-listen`, "Shared shapes and spellings", "JSON keys are camelCase" and "Only Ortie's own types are described" modified), repl (cairn/spec/repl.md: "Command grammar reuse" modified), provider-quirks (cairn/spec/provider-quirks.md: its three requirements modified), discovery (cairn/spec/discovery.md deleted, its 23 requirements removed). The auth, output, packaging and provider-quirks preambles no longer name the wizard. The active live-provider-tests change drops `wizard_discovers_fastmail`.

Verified: `cargo test --all-features` (19 unit tests and the integration suites), `cargo clippy --all-features --all-targets -D warnings`, `cargo check --no-default-features --features rustls-ring`, `cargo deny check`. By hand: a bare `ortie` prints the help, `configure` is an unknown subcommand, a missing configuration fails pointing at the samples in text and under `--json`, and a `--json` `auth get` hand-off from the Graph mail sample carries `redirect_uri=http://localhost:<port>/` and the Graph scopes, from the Google mail sample `http://127.0.0.1:<port>/`, `https://mail.google.com/` and `access_type=offline` (nothing sent to either provider). Release binary (fat LTO, stripped): 6,556,264 bytes before, 4,520,680 after (-2.04 MB, -31%); 177 crates in the normal dependency tree before, 141 after.

Left:

- The owner's live sign-ins from samples/google-pimalaya-mail.toml and samples/microsoft-pimalaya-outlook.toml on the test accounts.
- GitHub push protection may stop the `GOCSPX-` value of the Google samples: the owner allows it, public by design.
- The Google data notice, now owed by `auth get` before the browser opens, before the next release: its own change.
- The CLIs' wizards and pimalaya-cli's hand-off to `ortie configure`, which now fails: Ortie's release ships no earlier than the CLI releases that stop calling it.
