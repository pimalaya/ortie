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

- **Configuration wizard**: discovers your provider's grants and writes the account for you.
- **Dynamic client registration**: registers a public client on the spot, no provider console.
- **Authorization code grant**: browser sign-in, with a built-in server catching the redirection.
- **Device authorization grant**: a short code typed on another device, for hosts with no browser.
- **Client credentials grants**: headless machine tokens, by client secret or signed JWT assertion.
- **Manual completion**: finish a flow by hand when the redirection server cannot bind.
- **Token refresh**: on demand, or automatically when the token is read.
- **PKCE**: S256 by default, following the OAuth 2.1 posture.
- **Extra parameters**: provider-specific authorization parameters forwarded verbatim.
- **Token storage**: read and write tokens through your own shell commands.
- **Hooks**: a shell command or a desktop notification on issuance and refresh.
- **Persistent session**: unlock the secret store once, then answer token commands over stdin.
- **JSON output**: `--json` on every data command, for scripts.
- **TLS**: [rustls](https://crates.io/crates/rustls) with ring (`rustls-ring`, default) or aws (`rustls-aws`) crypto, or [native-tls](https://crates.io/crates/native-tls) (`native-tls`).

> [!TIP]
> Ortie is written in [Rust](https://www.rust-lang.org/) and uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate optional functionality. The default feature set is declared in [Cargo.toml](./Cargo.toml).

## Coverage

| RFC    | What is covered |
|--------|-----------------|
| [6749] | The OAuth 2.0 framework: authorization code and client credentials grants, access token issuance and refresh |
| [7523] | JWT client authentication on the client credentials grant: assertion signed with a private key, certificate `x5t` thumbprint (Microsoft certificate credentials) |
| [7636] | PKCE: the S256 and plain code challenges protecting the authorization code in transit |
| [7591] | Dynamic client registration: register a public client without any provider console |
| [8414] | Authorization server metadata: the wizard reads it to discover a provider's endpoints and registration endpoint |
| [8628] | Device authorization grant: device and user code request, token endpoint polling |

[6749]: https://www.rfc-editor.org/rfc/rfc6749
[7523]: https://www.rfc-editor.org/rfc/rfc7523
[7636]: https://www.rfc-editor.org/rfc/rfc7636
[7591]: https://www.rfc-editor.org/rfc/rfc7591
[8414]: https://www.rfc-editor.org/rfc/rfc8414
[8628]: https://www.rfc-editor.org/rfc/rfc8628

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

Run `ortie` with no command: it offers to generate a first account, which `ortie configure` does again later. From an email address, a domain or an issuer URL it discovers the grants your provider offers, walks you through the application, the scopes and the token storage, then appends the account to your configuration. What it cannot discover it does not ask for: the annotated [config.sample.toml](./config.sample.toml) is the reference for the rest. Run `ortie auth get` afterwards to authorize the account and store its first token.

A configuration is loaded from the first valid path among $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml and $HOME/.ortierc. Override it with `-c <PATH>` or `ORTIE_CONFIG=<PATH>`, `:`-separated to deep-merge several files on top of the first.

An OAuth 2.0 application is needed too. The wizard offers three, most preferred first: dynamic registration when the provider advertises it, a public application ([Thunderbird credentials](https://github.com/mozilla/releases-comm-central/blob/master/mailnews/base/src/OAuth2Providers.sys.mjs) cover most consumer providers), or your own, left as an empty `client-id` to fill in.

Ready-made blocks for common providers follow, for manual setups and for Microsoft Graph, which the wizard does not cover. Drop one under your `[accounts.<name>]` table and fill in the client credentials.

### Google

```toml
endpoints.authorization = "https://accounts.google.com/o/oauth2/v2/auth"
endpoints.token = "https://oauth2.googleapis.com/token"
scopes = ["https://www.googleapis.com/auth/carddav", "https://mail.google.com/"]
extras.access_type = "offline"
```

Use these endpoints, not the legacy `o/oauth2/auth` and `oauth2/v3/token` pair, which Google can reject at consent with "This app is blocked". Gmail and CardDAV being sensitive scopes, an unverified application of your own only works for accounts listed as test users; the Thunderbird one below is verified.

Contacts are split across two scopes that are not interchangeable: `auth/carddav` authorizes the CardDAV endpoint, `auth/contacts` the People API. Calendars are not split, CalDAV using the plain `auth/calendar` scope. A client id is verified for a fixed set of scopes, so asking the Thunderbird application for a People API scope fails at consent.

Public Thunderbird application:

```toml
client-id = "406964657835-aq8lmia8j95dhl1a2bvharmfk3t1hgqj.apps.googleusercontent.com"
client-secret.raw = "kSmqreRr0qwBWJgbf5Y-PjSU"
endpoints.redirection = "http://localhost"
```

For your [own application](https://developers.google.com/identity/protocols/oauth2), set `client-id` and `client-secret.raw` to your registered values.

### Microsoft (Outlook IMAP / SMTP)

```toml
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
```

Public Thunderbird application:

```toml
client-id = "9e5f94bc-e8a4-4e73-b8be-63364c29d753"
endpoints.redirection = "https://localhost"
```

### Microsoft Graph

The Thunderbird application above is registered for Outlook IMAP and SMTP, not for the Graph API. Graph tokens need Graph scopes from a client registered for Graph:

```toml
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
scopes = ["https://graph.microsoft.com/User.Read", "https://graph.microsoft.com/Mail.ReadWrite", "https://graph.microsoft.com/Mail.Send", "offline_access"]
```

Public Microsoft Graph PowerShell application:

```toml
client-id = "14d82eec-204b-4c2f-b7e8-296a70dab67e"
endpoints.redirection = "http://localhost"
```

Work or school (Entra ID) accounts receive a JWT the Graph API accepts. Personal accounts may get an opaque token it rejects with InvalidAuthenticationToken, so prefer a work or school account, or an application of your own.

### Fastmail

Fastmail advertises RFC 7591 dynamic registration, so bare `ortie` can register a client for you. Two specifics it fills in, worth knowing when writing the block by hand:

1. RFC 8707 resource: without a resource indicator, the authorize endpoint bounces the request with `invalid_target`, before any consent screen. Its value is the JMAP session URL.
2. Redirect: dynamic registration accepts only a reverse-DNS private-use scheme, `org.pimalaya.ortie://redirect`. No desktop browser routes it back, so `auth get` prints a manual `auth resume` command.

```toml
endpoints.authorization = "https://api.fastmail.com/oauth/authorize"
endpoints.token = "https://api.fastmail.com/oauth/refresh"
scopes = ["urn:ietf:params:oauth:scope:mail", "urn:ietf:params:oauth:scope:contacts", "urn:ietf:params:oauth:scope:calendars", "offline_access"]
extras.resource = "https://api.fastmail.com/jmap/session"
```

The wizard selects all four advertised scopes by default; trim them in the multi-select. The Thunderbird application it also offers covers Fastmail with a loopback redirect, avoiding the manual resume.

## Usage

Configure an account, authorize it, then read its token:

```sh
ortie configure                        # discover a provider and write the account
ortie auth get                         # authorize and store a first token
ortie auth resume <URI|DEVICE_CODE>    # finish a flow by hand
ortie token show                       # print the stored access token
ortie token refresh                    # force a refresh
ortie token inspect                    # print type, scopes and expiry
```

`ortie repl` answers those `token` commands from stdin instead, reading the secret store once so a keyring is unlocked one time rather than per call:

```sh
printf 'token show\n' | ortie repl
```

Every command and every flag is documented behind `--help`. `--json` switches data commands to machine-readable objects, and logs go to stderr, `--log-level <LEVEL>` and `--log-file <PATH>` setting their verbosity and destination.

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
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
