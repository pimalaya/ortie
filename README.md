# 🔑 Ortie [![Documentation](https://img.shields.io/docsrs/ortie?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/ortie/latest/ortie) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

Library and CLI to manage OAuth 2.0 tokens, written in Rust.

This repository ships three layers:

- Low-level **I/O-free** coroutines: no_std-friendly state machines that emit read/write requests for any runtime.
- Mid-level **blocking client** wrapping the coroutines over a `pimalaya-stream` connection.
- High-level **CLI** consuming the std client, configured through TOML.

## Table of contents

- [Features](#features)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Configuration](#configuration)
  - [Google](#google)
  - [Microsoft](#microsoft)
- [Usage](#usage)
  - [Library](#library)
  - [Request a new access token](#request-a-new-access-token)
  - [Refresh an access token](#refresh-an-access-token)
  - [Show an access token](#show-an-access-token)
  - [Debugging](#debugging)
- [Alternatives](#alternatives)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- **OAuth 2.0** Authorization Code Grant <sup>[rfc6749 #4.1](https://datatracker.ietf.org/doc/html/rfc6749#section-4.1)</sup> and refresh <sup>[rfc6749 #6](https://datatracker.ietf.org/doc/html/rfc6749#section-6)</sup>
- **PKCE** <sup>[rfc7636](https://datatracker.ietf.org/doc/html/rfc7636)</sup>
- **TLS** support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto (`rustls-ring` feature, default)
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)
- Fake HTTP **redirection server** during the interactive flow
- Shell command **storages** for reading and writing access tokens
- Shell command **hooks** on success and error of token issuance / refresh
- System notification **hooks** (requires `notify` feature)
- **JSON** output via `--json`

> [!TIP]
> Ortie is written in [Rust](https://www.rust-lang.org/) and uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate optional functionality. The default feature set is declared in [Cargo.toml](./Cargo.toml).

## Installation

### Pre-built binary

Ortie can be installed with the installer:

*As root:*

```text
curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | sudo sh
```

*As a regular user:*

```text
curl -sSL https://raw.githubusercontent.com/pimalaya/ortie/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/ortie/releases) section.

For a more up-to-date version than the latest release, check out the [releases](https://github.com/pimalaya/ortie/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. These pre-built binaries are built from the `master` branch.

> [!NOTE]
> Such binaries are built with the default cargo features. If you need specific features, please use another installation method.

### Cargo

```text
cargo install --locked ortie
```

For the git tip:

```text
cargo install --locked --git https://github.com/pimalaya/ortie.git
```

### Nix

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```text
nix profile install github:pimalaya/ortie
```

Or run without installing:

```text
nix run github:pimalaya/ortie
```

### Sources

```text
git clone https://github.com/pimalaya/ortie
cd ortie
nix run
```

## Configuration

Ortie does not yet ship a wizard: copy [config.sample.toml](./config.sample.toml) into one of the canonical paths below and edit it by hand.

A configuration is loaded from the first valid path among:

- `$XDG_CONFIG_HOME/ortie/config.toml`
- `$HOME/.config/ortie/config.toml`
- `$HOME/.ortierc`

Override the path with `-c <PATH>` or `ORTIE_CONFIG=<PATH>`; multiple paths can be passed at once, separated by `:`. The first one is the base and the rest are deep-merged on top.

You will also need a registered OAuth 2.0 application: either use a public application (Thunderbird credentials cover most consumer providers) or register your own. The first option is simpler.

*See public Thunderbird application credentials for various providers at [github.com/mozilla](https://github.com/mozilla/releases-comm-central/blob/master/mailnews/base/src/OAuth2Providers.sys.mjs).*

### Google

```toml
endpoints.authorization = "https://accounts.google.com/o/oauth2/auth?access_type=offline"
endpoints.token = "https://www.googleapis.com/oauth2/v3/token"
scopes = ["https://www.googleapis.com/auth/carddav", "https://mail.google.com"]
```

Using the public Thunderbird application:

```toml
client-id = "406964657835-aq8lmia8j95dhl1a2bvharmfk3t1hgqj.apps.googleusercontent.com"
client-secret.raw = "kSmqreRr0qwBWJgbf5Y-PjSU"
endpoints.redirection = "http://localhost"
```

Using your [own application](https://developers.google.com/identity/protocols/oauth2):

```toml
client-id = "<your-client-id>"
client-secret.raw = "<your-client-secret>"
```

### Microsoft

```toml
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
```

Using the public Thunderbird application:

```toml
client-id = "9e5f94bc-e8a4-4e73-b8be-63364c29d753"
endpoints.redirection = "https://localhost"
```

Using your [own application](https://learn.microsoft.com/en-us/exchange/client-developer/legacy-protocols/how-to-authenticate-an-imap-pop-smtp-application-by-using-oauth):

```toml
client-id = "<your-client-id>"
client-secret.raw = "<your-client-secret>"
```

## Usage

### Library

The low-level coroutines live under `ortie::authorization_code_grant`, `ortie::issue_access_token` and `ortie::refresh_access_token`; they emit `WantsRead` / `WantsWrite(bytes)` events the caller drives against any transport. The mid-level `ortie::client::OauthClient` wraps them with a blocking `pimalaya-stream` connection:

```rust,ignore
use ortie::client::OauthClient;

let mut client = OauthClient::new(&token_endpoint, &tls, &client_id);
let res = client.refresh_access_token(refresh_token, scopes)?;
```

A complete example using the authorization code grant flow lives in [examples/authorization_code_grant.rs](./examples/authorization_code_grant.rs).

### Request a new access token

```text
$ ortie auth get

Created authorization request with:
 - state: RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4
 - pkce: oJ-rEXNu9YzqpCWVIPOwD5KvMhLAT73dstk0jye8nZ6

Sending authorization request to your browser…
Spawning fake HTTP redirection server…
Waiting for redirection…
```

Follow the browser flow, then on success the terminal shows:

```text
Continue authorization process…
Access token successfully issued (expires in 1h)
```

If the redirection server cannot start (port permission denied, etc.), copy the URL you are redirected to and complete the flow manually:

```text
ortie auth resume \
  --state RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4 \
  --pkce oJ-rEXNu9YzqpCWVIPOwD5KvMhLAT73dstk0jye8nZ6 \
  https://localhost/?code=M.C521_BAY.2.U&state=RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4
```

### Refresh an access token

```text
$ ortie token refresh

Access token successfully refreshed (expires in 1h)
```

### Show an access token

```text
$ ortie token show

EwA4BOl3BAAUcDnR9grBJokeAHaUV8R3+rVHX+IAAQfw9oZLztQS8bo8NvyWmbs…
```

The `--auto-refresh` flag (or the `auto-refresh = true` config option) automatically refreshes expired tokens.

Inspect token metadata:

```text
$ ortie token inspect

Token type: bearer
Issued: 22h 51m 1s ago
Expires in: 52m 38s
With refresh token: true
With scope: https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send
```

### Debugging

The `--log-level <LEVEL>` flag controls log verbosity (`off`, `error`, `warn`, `info`, `debug`, `trace`). When omitted, `RUST_LOG` is consulted; it supports per-target filters (see the [env_logger](https://docs.rs/env_logger) docs). `RUST_BACKTRACE=1` enables the full error backtrace.

Logs go to stderr by default; redirect them with `--log-file <PATH>` or shell redirection:

```text
ortie token show --log-level debug --log-file /tmp/ortie.log
ortie token show --log-level trace 2>/tmp/ortie.log
```

## Alternatives

- [pizauth](https://github.com/ltratt/pizauth): daemon-oriented alternative
- [oama](https://github.com/pdobsan/oama): Haskell alternative
- [mutt_oauth2.py](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/mutt_oauth2.py): Python script alternative

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
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
