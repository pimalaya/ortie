# 🔑 Ortie [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage OAuth tokens

```sh
$ ortie token inspect --json
```

```json
{
  "tokenType": "Bearer",
  "issuedAt": "2026-11-21T08:00:00Z",
  "expiresAt": "2026-11-21T09:00:00Z",
  "withRefreshToken": true,
  "scope": "https://mail.google.com/ https://www.googleapis.com/auth/calendar"
}
```

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Google](#google)
  - [Microsoft](#microsoft)
  - [Fastmail](#fastmail)
- [Usage](#usage)
- [Alternatives](#alternatives)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Every common grant**: authorization code with PKCE, device code, client credentials by secret or signed JWT
- **Token lifecycle**: refreshed when read or on demand, inspected without printing it, revoked at the server
- **Your own storage**: tokens read and written through your commands, such as `pass` or `secret-tool`
- **Ready-made providers**: Google, Microsoft and Fastmail accounts to paste from this README
- **Account check** proving the token storage reads, the token is usable and every endpoint answers
- **REPL** unlocking the secret store once per session, and **MCP server** never handing a token to the agent
- **JSON output** of every command, described by `ortie json-schema`
- **TLS** via Rustls (`rustls-ring` default, `rustls-aws`) or Native TLS (`native-tls`), client certificates included

## RFC coverage

Ortie follows the [OAuth 2.1](https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/) draft: no implicit nor password grant, PKCE S256 by default, `pkce = "plain"` or `false` for servers that need it.

| RFC    | What is covered                                                                                              |
|--------|--------------------------------------------------------------------------------------------------------------|
| [6749] | The OAuth 2.0 framework: authorization code and client credentials grants, access token issuance and refresh |
| [7009] | Token revocation: the refresh token, or the access token when none is stored                                 |
| [7523] | JWT client authentication: an assertion signed with a private key, carrying the certificate thumbprint       |
| [7636] | PKCE: the S256 and plain code challenges protecting the authorization code in transit                        |
| [8252] | Native apps: a loopback redirection naming no port gets a random one                                         |
| [8628] | Device authorization grant: device and user code request, token endpoint polling                             |
| [8707] | Resource indicators: the `resource` parameter a server bound to one resource requires                        |
| [9700] | Security best current practice: the refreshed token stored at once, the previous refresh token kept if none rotates |

[6749]: https://www.rfc-editor.org/rfc/rfc6749
[7009]: https://www.rfc-editor.org/rfc/rfc7009
[7523]: https://www.rfc-editor.org/rfc/rfc7523
[7636]: https://www.rfc-editor.org/rfc/rfc7636
[8252]: https://www.rfc-editor.org/rfc/rfc8252
[8628]: https://www.rfc-editor.org/rfc/rfc8628
[8707]: https://www.rfc-editor.org/rfc/rfc8707
[9700]: https://www.rfc-editor.org/rfc/rfc9700

## Installation

<details>
  <summary>Debian, Ubuntu</summary>

  Ortie is not in the official repositories yet, but the signed APT repository of [pkg.pimalaya.org](https://pkg.pimalaya.org) ships the latest release with its man pages and shell completions. Add the repository once:

  ```sh
  sudo install -d -m 0755 /etc/apt/keyrings
  curl -fsSL https://pkg.pimalaya.org/pimalaya.asc | sudo tee /etc/apt/keyrings/pimalaya.asc >/dev/null
  sudo tee /etc/apt/sources.list.d/pimalaya.sources >/dev/null <<EOF
  Types: deb
  URIs: https://pkg.pimalaya.org/apt
  Suites: stable
  Components: main
  Signed-By: /etc/apt/keyrings/pimalaya.asc
  EOF
  sudo apt update
  ```

  Then install Ortie:

  ```sh
  sudo apt install ortie
  ```
</details>

<details>
  <summary>Fedora, RHEL, Rocky Linux, AlmaLinux</summary>

  Ortie is not in the official repositories yet, but the signed DNF repository of [pkg.pimalaya.org](https://pkg.pimalaya.org) ships the latest release with its man pages and shell completions:

  ```sh
  sudo curl -fsSL -o /etc/yum.repos.d/pimalaya.repo https://pkg.pimalaya.org/pimalaya.repo
  sudo dnf install ortie
  ```
</details>

<details>
  <summary>Homebrew (macOS, Linux)</summary>

  Ortie is not in homebrew-core yet, but the [pimalaya/tap](https://github.com/pimalaya/homebrew-tap) formula installs the latest release with its man pages and shell completions:

  ```sh
  brew install pimalaya/tap/ortie
  ```
</details>

<details>
  <summary>Pre-built binary</summary>

  For other systems, Ortie can be installed with the installer:

  *As root:*

  ```sh
  curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | sudo sh
  ```

  *As a regular user:*

  ```sh
  curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | PREFIX=~/.local sh
  ```

  These commands install the latest [release](https://github.com/pimalaya/ortie/releases), built with the default cargo features. For a build of `master`, check out the [releases](https://github.com/pimalaya/ortie/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section.
</details>

<details>
  <summary>Cargo</summary>

  ```sh
  cargo install --locked ortie
  ```

  For a build of `master`:

  ```sh
  cargo install --locked --git https://github.com/pimalaya/ortie.git
  ```
</details>

<details>
  <summary>Nix</summary>

  ```sh
  nix profile install github:pimalaya/ortie
  ```
</details>

<details>
  <summary>Sources</summary>

  Requires [Rust](https://www.rust-lang.org/tools/install) 1.89 or later:

  ```sh
  git clone https://github.com/pimalaya/ortie
  cd ortie
  cargo build --release --locked
  ```

  The binary lands at target/release/ortie.
</details>

## Configuration

A TOML configuration is loaded from every existing path among the following, each one deep-merged on top of the ones before:

- `$XDG_CONFIG_HOME/ortie/config.toml`
- `$HOME/.config/ortie/config.toml`
- `$HOME/.ortierc`

Override the path with `-c <PATH>`, repeated to merge several files.

> [!TIP]
> See [config.sample.toml](./config.sample.toml) for a complete configuration of one example account with commented fields; rename the account and uncomment what you need, or start from a provider below.

### Google

Pimalaya's Google application is not released yet: fill `client-id` and `client-secret` with a Desktop client of your own, whose refresh tokens expire after 7 days while its consent screen stays in testing.

```toml
[accounts.google]

client-id = ""
client-secret.raw = ""
extras.access_type = "offline"
endpoints.authorization = "https://accounts.google.com/o/oauth2/v2/auth"
endpoints.token = "https://oauth2.googleapis.com/token"
endpoints.revocation = "https://oauth2.googleapis.com/revoke"
scopes = [
  "https://mail.google.com/",
  "https://www.googleapis.com/auth/calendar",
  "https://www.googleapis.com/auth/calendar.events",
  "https://www.googleapis.com/auth/calendar.calendarlist.readonly",
  "https://www.googleapis.com/auth/contacts",
  "https://www.googleapis.com/auth/carddav",
]

storage.read.command = "pass show ortie/google"
storage.write.command = "pass insert -m -f ortie/google"
```

### Microsoft

Both accounts run on Pimalaya's own Entra application. Entra issues a token for one resource, so Graph and IMAP/SMTP take one account each, and the redirection stays `http://localhost` (any port, never `127.0.0.1`).

```toml
[accounts.microsoft-graph]

client-id = "ba9b19e1-973e-4d7c-aed9-848bd2fee385"
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
endpoints.redirection = "http://localhost"
scopes = [
  "offline_access",
  "https://graph.microsoft.com/Mail.ReadWrite",
  "https://graph.microsoft.com/Mail.Send",
  "https://graph.microsoft.com/Calendars.ReadWrite",
  "https://graph.microsoft.com/Contacts.ReadWrite",
]

storage.read.command = "pass show ortie/microsoft-graph"
storage.write.command = "pass insert -m -f ortie/microsoft-graph"

[accounts.microsoft-imap-smtp]

client-id = "ba9b19e1-973e-4d7c-aed9-848bd2fee385"
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
endpoints.redirection = "http://localhost"
scopes = [
  "offline_access",
  "https://outlook.office.com/IMAP.AccessAsUser.All",
  "https://outlook.office.com/SMTP.Send",
]

storage.read.command = "pass show ortie/microsoft-imap-smtp"
storage.write.command = "pass insert -m -f ortie/microsoft-imap-smtp"
```

### Fastmail

The account runs on Pimalaya's registered client. Fastmail refuses the request without the JMAP session URL in `resources`, and any `extras` but `login_hint`.

```toml
[accounts.fastmail]

client-id = "df6e41ae"
endpoints.authorization = "https://api.fastmail.com/oauth/authorize"
endpoints.token = "https://api.fastmail.com/oauth/refresh"
endpoints.revocation = "https://api.fastmail.com/oauth/revoke"
endpoints.redirection = "http://localhost/"
resources = ["https://api.fastmail.com/jmap/session"]
scopes = [
  "offline_access",
  "urn:ietf:params:oauth:scope:mail",
  "urn:ietf:params:oauth:scope:contacts",
  "urn:ietf:params:oauth:scope:calendars",
]

storage.read.command = "pass show ortie/fastmail"
storage.write.command = "pass insert -m -f ortie/fastmail"
```

## Usage

**Step 1**: once your first account is configured, sign in: open the link Ortie prints, it catches the redirection and stores the token:

```sh
ortie auth get
```

**Step 2**: check that the storage reads, the token is usable and every endpoint answers:

```sh
ortie account check
```

**Step 3**: print the token, refreshed when expired, for the secret command of your mail client:

```sh
ortie token show
```

Every command runs on the default account unless `-a` names another, and its `--help` details its flags, exit codes and JSON output. More use cases:

```sh
# Inspect the stored token without printing it
ortie token inspect

# Print an account as loaded, secrets redacted
ortie account show

# Revoke the token at the server
ortie token revoke

# Unlock the secret store once for a whole session
ortie repl

# Serve the token lifecycle to an AI agent, never the token itself
ortie mcp --allow-write
```

## Alternatives

- [pizauth](https://github.com/ltratt/pizauth): a daemon holding tokens in memory and refreshing them in the background
- [oama](https://github.com/pdobsan/oama): a Haskell tool for mail clients, storing tokens in the keyring or GPG-encrypted files
- [mutt_oauth2.py](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/mutt_oauth2.py): Mutt's Python script, keeping one token in a GPG-encrypted file

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- 2026 → 2027: [NGI Zero Commons Fund](https://nlnet.nl/project/Pimalaya-pimdir/)

This program is part of Pimalaya, free software funded entirely by grants and donations. If you find it useful, consider [sponsoring](https://pimalaya.org/sponsor/) its development:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/pimalaya)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/pimalaya)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/pimalaya)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/u/gh/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
