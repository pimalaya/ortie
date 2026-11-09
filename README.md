# 🔑 Ortie [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage OAuth 2.0 tokens, written in Rust

## Table of contents

- [Features](#features)
- [Coverage](#coverage)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Alternatives](#alternatives)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Ready-made accounts**: one sample per use case, Google and Microsoft through Pimalaya's own applications.
- **Authorization code grant**: browser sign-in, with a built-in server catching the redirection.
- **Device authorization grant**: a short code typed on another device, for hosts with no browser.
- **Client credentials grants**: headless machine tokens, by client secret or signed JWT assertion.
- **Manual completion**: finish a handed-off flow with `auth listen`, or by hand with `auth resume`.
- **Token refresh**: on demand, or automatically when the token is read.
- **Token revocation**: retire a token at the authorization server.
- **PKCE**: S256 by default, following the OAuth 2.1 posture.
- **Extra parameters**: provider-specific authorization parameters forwarded verbatim.
- **Token storage**: read and write tokens through your own shell commands.
- **Hooks**: a shell command or a desktop notification on issuance and refresh.
- **Persistent session**: unlock the secret store once, then answer token commands over stdin.
- **MCP server**: an agent drives the grants through `ortie mcp`, and no tool returns a token.
- **JSON output**: `--json` on every data command, its payload described by `ortie json-schema`.
- **TLS**: [rustls](https://crates.io/crates/rustls) with ring (`rustls-ring`, default) or aws (`rustls-aws`) crypto, or [native-tls](https://crates.io/crates/native-tls) (`native-tls`).

> [!TIP]
> Ortie is written in [Rust](https://www.rust-lang.org/) and uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate optional functionality. The default feature set is declared in [Cargo.toml](./Cargo.toml).

## Coverage

| RFC    | What is covered |
|--------|-----------------|
| [6749] | The OAuth 2.0 framework: authorization code and client credentials grants, access token issuance and refresh |
| [7009] | Token revocation: `token revoke` retires the refresh token, or the access token when none is stored |
| [7523] | JWT client authentication on the client credentials grant: assertion signed with a private key, certificate `x5t` thumbprint (Microsoft certificate credentials) |
| [7636] | PKCE: the S256 and plain code challenges protecting the authorization code in transit |
| [8628] | Device authorization grant: device and user code request, token endpoint polling |
| [8707] | Resource indicators: the `resource` parameter a server bound to one resource requires |

[6749]: https://www.rfc-editor.org/rfc/rfc6749
[7009]: https://www.rfc-editor.org/rfc/rfc7009
[7523]: https://www.rfc-editor.org/rfc/rfc7523
[7636]: https://www.rfc-editor.org/rfc/rfc7636
[8628]: https://www.rfc-editor.org/rfc/rfc8628
[8707]: https://www.rfc-editor.org/rfc/rfc8707

### OAuth 2.1

Ortie follows the OAuth 2.1 draft and the security best current practice (RFC 9700):

- Grants: authorization code, device, and client credentials authenticated by secret or signed JWT. The implicit and resource owner password grants, removed by OAuth 2.1, are not implemented.
- PKCE: S256 on every authorization code flow by default, `pkce = "plain"` or `false` being escape hatches for servers that need them.
- Redirects: a configured redirection URI is sent as written, for the server to match exactly. Without one, Ortie binds `http://127.0.0.1` on a random port, the loopback exception that lets the port vary (RFC 8252 section 7.3); a configured `http` loopback naming no port, such as `http://localhost`, gets a random port the same way.
- Refresh rotation: a refreshed token is stored as soon as the server answers, keeping the previous refresh token when no rotated one comes back. When storing fails, the error says so, and a REPL session keeps the new tokens.

## Installation

### Pre-built binary

Ortie can be installed with the installer:

*As root:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | sudo sh
```

*As a regular user:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/ortie/releases) section.

For a more up-to-date version than the latest release, check out the [releases](https://github.com/pimalaya/ortie/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. These pre-built binaries are built from the master branch.

> [!NOTE]
> Such binaries are built with the default cargo features. If you need specific features, please use another installation method.

### Cargo

```sh
cargo install --locked ortie
```

For the git tip:

```sh
cargo install --locked --git https://github.com/pimalaya/ortie.git
```

### Nix

If you have the [Flakes](https://wiki.nixos.org/wiki/Flakes) feature enabled:

```sh
nix profile install github:pimalaya/ortie
```

Or run without installing:

```sh
nix run github:pimalaya/ortie
```

### Sources

```sh
git clone https://github.com/pimalaya/ortie
cd ortie
nix run
```

## Configuration

A configuration is loaded from the first valid path among $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml and $HOME/.ortierc. Override it with `-c <PATH>` or `ORTIE_CONFIG=<PATH>`, `:`-separated to deep-merge several files on top of the first.

Start from a sample: each file of [samples](./samples) is one complete account, ready to append to your configuration. Then run `ortie auth get -a <NAME>` to authorize it and store its first token. The annotated [config.sample.toml](./config.sample.toml) documents every field, storage commands and hooks included.

| Sample | Application | For |
|---|---|---|
| [google-pimalaya-mail.toml](./samples/google-pimalaya-mail.toml) | Pimalaya | Gmail over IMAP, SMTP or the Gmail API |
| [google-pimalaya-calendar.toml](./samples/google-pimalaya-calendar.toml) | Pimalaya | Google Calendar API |
| [google-pimalaya-contacts.toml](./samples/google-pimalaya-contacts.toml) | Pimalaya | Google People API |
| [microsoft-pimalaya-outlook.toml](./samples/microsoft-pimalaya-outlook.toml) | Pimalaya | Outlook IMAP and SMTP |
| [microsoft-pimalaya-graph-mail.toml](./samples/microsoft-pimalaya-graph-mail.toml) | Pimalaya | Microsoft Graph mail |
| [microsoft-pimalaya-graph-calendar.toml](./samples/microsoft-pimalaya-graph-calendar.toml) | Pimalaya | Microsoft Graph calendars |
| [microsoft-pimalaya-graph-contacts.toml](./samples/microsoft-pimalaya-graph-contacts.toml) | Pimalaya | Microsoft Graph contacts |
| [google-own-application.toml](./samples/google-own-application.toml) | your own | CalDAV, CardDAV, any other Google scope |
| [microsoft-own-application.toml](./samples/microsoft-own-application.toml) | your own | an organisation refusing Pimalaya's application |
| [fastmail-own-application.toml](./samples/fastmail-own-application.toml) | your own, registered by hand | Fastmail over JMAP |

Google scopes can share one account, a single token serving every API. Microsoft scopes cannot: Entra issues a token for one resource, so Outlook IMAP and SMTP and the Graph API need separate accounts, while the Graph samples can be merged into one.

## Usage

Append a sample to your configuration, authorize the account, then read its token:

```sh
ortie auth get                         # authorize and store a first token
ortie auth listen --state=… --pkce=… --redirect-uri=…  # catch a handed-off redirection
ortie auth resume <URI|DEVICE_CODE>    # finish a flow by hand
ortie token show                       # print the stored access token
ortie token refresh                    # force a refresh
ortie token inspect                    # print type, scopes and expiry
ortie token revoke                     # revoke the token at the server
```

`ortie repl` answers those `token` commands from stdin instead, reading the secret store once so a keyring is unlocked one time rather than per call:

```sh
printf 'token show\n' | ortie repl
```

`ortie mcp` runs a [Model Context Protocol](https://modelcontextprotocol.io) server over stdio for one account, so an agent can obtain and renew its token. `auth_get` answers the authorization URI to open, or the user code to enter, and the server keeps the state, the PKCE verifier and the device code to itself: it catches the redirection or polls the device code, then stores the token once the user consents, and `auth_status` reports the outcome. `auth_resume` takes the redirected URL when no listener can catch it, and `token_inspect` and `token_refresh` work on the stored token. No tool returns the token, which applications keep reading through `ortie token show`; `token_revoke` is listed only with `--allow-revoke`.

```json
{
  "mcpServers": {
    "ortie": { "command": "ortie", "args": ["mcp", "--account", "work"] }
  }
}
```

Every command and every flag is documented behind `--help`. `--json` switches data commands to machine-readable objects, `ortie json-schema --dir <DIR>` writing the JSON Schema of each, and logs go to stderr, with `--log-level <LEVEL>` and `--log-file <PATH>` setting their verbosity and destination.

## Alternatives

- [pizauth](https://github.com/ltratt/pizauth): daemon-oriented alternative
- [oama](https://github.com/pdobsan/oama): Haskell alternative
- [mutt_oauth2.py](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/mutt_oauth2.py): Python script alternative

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
