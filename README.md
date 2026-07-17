# 🔑 Ortie [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

CLI to manage OAuth tokens

## Table of contents

- [Features](#features)
- [Coverage](#coverage)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Configuration](#configuration)
  - [Google](#google)
  - [Microsoft (Outlook IMAP / SMTP)](#microsoft-outlook-imap--smtp)
  - [Microsoft Graph](#microsoft-graph)
  - [Fastmail](#fastmail)
- [Usage](#usage)
- [Alternatives](#alternatives)
- [AI disclosure](#ai-disclosure)
- [License](#license)
- [Social](#social)
- [Contributing](#contributing)
- [Sponsoring](#sponsoring)

## Features

- **Account discovery wizard**: run bare, it finds the OAuth 2.0 grants reachable for an email address and prints a ready-to-append account configuration.
- **Dynamic client registration**: register a public client on the spot, with no provider console, when the provider advertises it.
- **Authorization code grant**: sign in through the browser, with a built-in redirection server that captures the callback.
- **Manual flow completion**: finish a flow by hand, from the URL your browser was sent to, when the redirection server cannot bind.
- **Token refresh**: renew an expired access token from its refresh token, on demand or automatically when reading it.
- **PKCE**: enabled with the S256 method by default, following the OAuth 2.1 posture; opt out for servers that reject it.
- **Extra authorization parameters**: forward provider-specific knobs (Google offline access, login hints, resource indicators) verbatim.
- **Token storage**: read and write tokens through your own shell commands, wiring into any credential manager.
- **Hooks**: run a shell command or raise a system notification on token issuance and refresh, split by outcome.
- **JSON output**: switch discovery and token commands to machine-readable output for scripts.
- Full standard, blocking client with **TLS** support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto (requires `rustls-ring` feature, enabled by default)
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)

> [!TIP]
> Ortie uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate optional functionality; the default set is declared in Cargo.toml.

## Coverage

| RFC    | What is covered |
|--------|-----------------|
| [6749] | The OAuth 2.0 framework: authorization code grant, access token issuance and refresh |
| [7636] | PKCE: the S256 and plain code challenges protecting the authorization code in transit |
| [7591] | Dynamic client registration: register a public client without any provider console |
| [8414] | Authorization server metadata: the wizard reads it to discover a provider's endpoints and registration endpoint |

[6749]: https://www.rfc-editor.org/rfc/rfc6749
[7636]: https://www.rfc-editor.org/rfc/rfc7636
[7591]: https://www.rfc-editor.org/rfc/rfc7591
[8414]: https://www.rfc-editor.org/rfc/rfc8414

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

Run ortie with no argument to launch the discovery wizard: it asks for an email address (or a server or issuer URI), discovers the OAuth 2.0 services reachable for it, and prints a complete account fragment as valid TOML on stdout, its guidance embedded as comments. Ortie never writes your configuration itself; the prompts render on stderr, so appending the fragment is a one-liner, `ortie >> ~/.config/ortie/config.toml`.

A configuration is loaded from the first valid path among:

- `$XDG_CONFIG_HOME/ortie/config.toml`
- `$HOME/.config/ortie/config.toml`
- `$HOME/.ortierc`

Override the path with `-c <PATH>` or `ORTIE_CONFIG=<PATH>`; multiple paths can be passed at once, separated by :. The first one is the base and the rest are deep-merged on top. The full field reference lives in [config.sample.toml](./config.sample.toml); ready-made per-provider blocks follow below.

You may also need a registered OAuth 2.0 application. The wizard offers three ways, most preferred first: dynamic registration when your provider advertises it (Fastmail does), a public application (Thunderbird credentials cover most consumer providers), or your own registration. Public Thunderbird credentials for various providers are listed at [github.com/mozilla](https://github.com/mozilla/releases-comm-central/blob/master/mailnews/base/src/OAuth2Providers.sys.mjs).

Ready-made configuration blocks for common providers follow. The discovery wizard fills most of these in for you; they are kept here for manual setups and for Microsoft Graph, which the wizard does not cover. Drop the relevant block under your `[accounts.<name>]` table and fill in the client credentials.

### Google

```toml
endpoints.authorization = "https://accounts.google.com/o/oauth2/v2/auth"
endpoints.token = "https://oauth2.googleapis.com/token"
scopes = ["https://www.googleapis.com/auth/carddav", "https://mail.google.com"]
extras.access_type = "offline"
```

Use these current endpoints, not the legacy `o/oauth2/auth` / `www.googleapis.com/oauth2/v3/token` pair, which Google can reject at consent with "This app is blocked"; the discovery wizard already fills the current ones. Gmail and CardDAV are sensitive scopes, so an unverified own application only works for accounts listed as test users on its OAuth consent screen; the public Thunderbird application below is verified.

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

The Thunderbird application above is registered for Outlook IMAP / SMTP, not for the Graph API. To mint Graph tokens (for example Himalaya's msgraph backend), request Graph scopes from a client registered for Graph:

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

Work or school (Entra ID) accounts receive a JWT the Graph API accepts; personal Microsoft accounts may be issued an opaque token the API rejects with InvalidAuthenticationToken, so prefer a work or school account, or your own registered application.

### Fastmail

Fastmail advertises RFC 7591 dynamic registration, so bare `ortie` can register a client for you. Two Fastmail specifics the wizard does not yet fill into the fragment, so add them by hand:

1. RFC 8707 resource: Fastmail's authorize endpoint rejects the request with `invalid_target` (no consent screen, an instant bounce) unless a resource indicator is present. Its value is the JMAP session URL.
2. Redirect: Fastmail's dynamic registration accepts only a reverse-DNS private-use scheme (it refuses http and loopback), so the wizard pins `endpoints.redirection = "org.pimalaya.ortie://redirect"`. A desktop browser cannot route that scheme back to ortie, so `auth get` prints a manual `auth resume` command to finish the flow by hand.

```toml
endpoints.authorization = "https://api.fastmail.com/oauth/authorize"
endpoints.token = "https://api.fastmail.com/oauth/refresh"
scopes = ["urn:ietf:params:oauth:scope:mail", "urn:ietf:params:oauth:scope:contacts", "urn:ietf:params:oauth:scope:calendars", "offline_access"]
extras.resource = "https://api.fastmail.com/jmap/session"
```

The wizard selects all four advertised scopes by default (Fastmail cannot complete on a desktop anyway; trim them in the scope multi-select if you want). The pre-registered Thunderbird application offered by the wizard covers Fastmail with a loopback redirect instead, avoiding the manual resume; see [docs/providers.md](./docs/providers.md) and [docs/discovery-layering.md](./docs/discovery-layering.md) for why dynamic registration forces the private-use scheme.

## Usage

Every command and subcommand is documented through --help. The common flows:

```sh
ortie                       # discover an account and print a config fragment
ortie auth get              # request a first access token through the browser
ortie auth resume <url>     # finish the flow by hand when the redirection fails
ortie token show            # print the stored access token
ortie token refresh         # force a refresh
ortie token inspect         # show token metadata (type, scopes, expiry)
```

Logs go to stderr; `--log-level` and `--log-file` control verbosity and destination, and `--json` switches output to machine-readable objects.

## Alternatives

- [pizauth](https://github.com/ltratt/pizauth): daemon-oriented alternative
- [oama](https://github.com/pdobsan/oama): Haskell alternative
- [mutt_oauth2.py](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/mutt_oauth2.py): Python script alternative

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit. Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 16/07/2026

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Contributing

Contributions are welcome: start with [CONTRIBUTING.md](./CONTRIBUTING.md), which opens with the Pimalaya-wide guides to read first.

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
