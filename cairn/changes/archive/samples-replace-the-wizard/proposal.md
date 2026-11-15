---
cairn: change
id: samples-replace-the-wizard
status: landed
created: 2026-11-08
---

# Samples replace the wizard

## Why

The wizard sits between two audiences it does not serve well: power users write their account from config.sample.toml, and users who need a guided setup belong in the Pimalaya apps, not in a command-line OAuth tool. Between them it costs the most of anything in Ortie: 3,126 lines (src/wizard.rs and src/wizard/), 23 of the 89 spec requirements (cairn/spec/discovery.md) plus 7 in cairn/spec/config.md, ten changes since 2026-07-13 (the eight `wizard-*`, first-run-wizard-frame, the-wizard-takes-its-input), and still a source of confusion (ortie #13, a first run that ended on a fragment the user could not complete).

The owner decided on 2026-11-08 to drop the wizards across the family: the four CLIs lose theirs, and with them the hand-off to `ortie configure`. Discovery itself stays in io-pim-discovery, which the Android app uses and which ships its own command-line interface for whoever wants discovery from a terminal.

What the wizard knew for the common cases is fixed data: Google's and Microsoft's endpoints, Pimalaya's client ids, the scopes each use needs, the provider quirks. Files hold fixed data better than prompts, and a test that parses each file keeps it current with the configuration.

It also takes dependencies the next versions want gone, the command-line interfaces moving to plain text and JSON output with no terminal handling: io-pim-discovery (DNS, public suffix list, XML), and through pimalaya-cli's `prompt`, `wizard` and `spinner` features, inquire, crossterm and ctrlc. Measured on himalaya on 2026-11-08 (release profile, fat LTO, stripped): its `wizard` feature weighs 1.90 MB of a 20.3 MB binary, crossterm and inquire another 155 KB across 17 crates. Ortie's own numbers go in the log.

## What

- **Removed:** the `configure` command (alias `wizard`) and its `--service` and `--name` hand-off; the offer a missing configuration raised from a bare `ortie` and from any command needing an account; the welcome; the `wizard` cargo feature; the `ortie-configure` schema key; dynamic client registration (RFC 7591), which only ever ran inside the wizard.
- **Dependencies leaving:** io-pim-discovery, and pimalaya-cli's `prompt`, `wizard` and `spinner` features (inquire, crossterm, ctrlc). Ortie keeps pimalaya-cli's `terminal` feature only, and prints no table.
- **A bare `ortie` shows the help**, configured or not. A missing configuration fails like every other account-resolution failure, naming the path it looked for, and points at the samples.
- **Added: samples/, one complete `[accounts.<name>]` table per use case**, ready to append to the configuration. config.sample.toml stays the reference of every field. The provider blocks of the README move into these files, and the README links them.

  | Sample | Application | Scopes | For |
  |---|---|---|---|
  | samples/google-pimalaya-mail.toml | Pimalaya (Google Desktop client) | `https://mail.google.com/` | IMAP, SMTP and the Gmail API: himalaya, neverest |
  | samples/google-pimalaya-calendar.toml | Pimalaya | `calendar.events`, `calendar.calendarlist.readonly` | Calendar API: calendula |
  | samples/google-pimalaya-contacts.toml | Pimalaya | `contacts` | People API: cardamum |
  | samples/microsoft-pimalaya-outlook.toml | Pimalaya (Entra) | `offline_access`, Outlook `IMAP.AccessAsUser.All`, `SMTP.Send` | IMAP and SMTP |
  | samples/microsoft-pimalaya-graph-mail.toml | Pimalaya | `offline_access`, `Mail.ReadWrite`, `Mail.Send` | Graph mail: himalaya, neverest |
  | samples/microsoft-pimalaya-graph-calendar.toml | Pimalaya | `offline_access`, `Calendars.ReadWrite` | Graph calendars: calendula |
  | samples/microsoft-pimalaya-graph-contacts.toml | Pimalaya | `offline_access`, `Contacts.ReadWrite` | Graph contacts: cardamum |
  | samples/google-own-application.toml | the user's own | IMAP, SMTP, CalDAV, CardDAV scopes | a registration of one's own, with how to make it |
  | samples/microsoft-own-application.toml | the user's own | Outlook IMAP, SMTP | a registration of one's own, a tenant that refuses Pimalaya's |
  | samples/fastmail-own-application.toml | the user's own | the JMAP session's | Fastmail, registered by hand (the registration request in a comment), RFC 8707 resource, private-use redirection |

  The calendar samples leave the Tasks scopes out: calendula's Google and Graph backends handle no todos (checked 2026-11-08). The Microsoft samples pin `endpoints.redirection = "http://localhost"` (Pimalaya's registration refuses `127.0.0.1`, any port accepted). Each Microsoft sample stays on one resource, since Entra issues a token for one resource only; the Google scopes may be merged into one account, one token serving every API. Every sample stores its token with `pass`, a comment naming the Secret Service and macOS Keychain lines of config.sample.toml.
- **Every sample is parsed in the test suite** through `Config::from_paths`, the loader `-c` uses, and must hold exactly one account, so a configuration change that breaks a sample fails the build. Whether the providers still accept them is the live tests' job (live-provider-tests), which can take the samples as their fixtures.
- **Kept from pimalaya-applications-lead (landed 2026-11-08):** the ephemeral port a port-less `http` loopback redirection binds, in `auth get` and the MCP server. The rest of it (known applications, `--service`, scope narrowing) leaves with the wizard, its data moving into the samples.
- **Spec drift fixed on the way:** "Commands carrying a schema" never listed `ortie-auth-listen`, which src/json_schema.rs registers. Four other requirements still named the wizard, Thunderbird or io-pim-discovery ("JSON keys are camelCase", "Only Ortie's own types are described", and the three provider quirks) and are reworded.

## Decisions

Accepted by the owner on 2026-11-08 ("Go for it").

- **The Google client secret is written in its sample as is.** GitHub push protection may flag a `GOCSPX-` value: the owner allows it once, as public by design (RFC 8252 section 8.5, Google's own installed-application documentation). Weighed and set aside: an `application = "pimalaya"` key resolving built-in credentials, which would let a rotated secret reach users through an upgrade, but hides what is sent; revisit if the secret is ever rotated.
- **Dynamic registration goes, not moved into a command.** Fastmail is the one provider of the test set advertising it, and accepts only a private-use redirection; its sample shows the registration request to run once by hand. Pimalaya's tools reach Fastmail without Ortie too, through an app password or a JMAP API token.

## Not in this change

- The Google data notice owed before a sign-in with Pimalaya's Google client (android cairn/changes/oauth-verification, tasks section 1): planned for the wizard, it moves to `auth get`, before the browser opens. Its own change, owed before the next release; open there is how a `--json` hand-off, `auth listen` and the MCP `auth_get` show it and take the user's agreement.
- The CLIs' wizards and pimalaya-cli's hand-off (cli/src/wizard/ortie.rs): their own changes. Ortie's release without `configure` must not ship before the CLI releases that stop calling it.
- A device grant sample with Pimalaya's Entra application: "Allow public client flows" on the registration is not probed.
- A Pimalaya client at Fastmail.
