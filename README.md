# 🔑 Ortie [![Matrix](https://img.shields.io/matrix/pimalaya:matrix.org?color=success&label=chat)](https://matrix.to/#/#pimalaya:matrix.org)

CLI to manage OAuth 2.0 access tokens

## Table of contents

- [Features](#features)
- [Usage](#usage)
  - [Request new access token](#request-new-access-token)
  - [Refresh access token](#refresh-access-token)
  - [Show access token](#show-access-token)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Google](#google)
  - [Microsoft](#microsoft)
- [FAQ](#faq)
- [Sponsoring](#sponsoring)

## Features

- **OAuth 2.0** support:
  - Authorization Code Grant <sup>[rfc6749 #4.1](https://datatracker.ietf.org/doc/html/rfc6749#section-4.1)</sup>
  - Refreshing an access token <sup>[rfc6749 #6](https://datatracker.ietf.org/doc/html/rfc6749#section-6)</sup>
- **PKCE** support <sup>[rfc7636](https://datatracker.ietf.org/doc/html/rfc7636)<sup>
- Native TLS support via [native-tls](https://crates.io/crates/native-tls) crate (requires `native-tls` feature)
- Rust TLS support via [rustls](https://crates.io/crates/rustls) crate with:
  - AWS crypto support (requires `rustls-aws` feature)
  - Ring crypto support (requires `rustls-ring` feature)
- Fake HTTP **redirection server**
- Shell command and keyring **storages** (requires `command` and `keyring` features)
- Shell command and system notification **hooks** (requires `command` and `notify` features)
- **JSON** support with `--json`

*Ortie CLI is written in [Rust](https://www.rust-lang.org/), and relies on [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to enable or disable functionalities. Default features can be found in the `features` section of the [`Cargo.toml`](https://github.com/pimalaya/ortie/blob/master/Cargo.toml#L18), or on [docs.rs](https://docs.rs/crate/ortie/latest/features).*

## Usage

### Request new access token

```
$ ortie auth get

Created authorization request with:
 - state: RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4
 - pkce: oJ-rEXNu9YzqpCWVIPOwD5KvMhLAT73dstk0jye8nZ6

Sending authorization request to your browser…
Spawning fake HTTP redirection server…
Waiting for redirection…
```

Go to your browser, follow the instructions, then you should see:

```
Authorization succeeded!
```

Go back to your terminal, you should see:

```
Continue authorization process…
Access token successfully issued (expires in 1h)
```

In case the redirections fails, for example:

```
$ ortie auth get

Created authorization request with:
 - state: RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4
 - pkce: oJ-rEXNu9YzqpCWVIPOwD5KvMhLAT73dstk0jye8nZ6

Sending authorization request to your browser…
Spawn fake HTTP redirection server…
Error: Permission denied (os error 13)
```

Go to your browser, follow the instructions, then copy the URL you are redirected to (it should fail since the fake HTTP redirection server did not start).

Go back to your terminal, and complete the authorization flow:

```
ortie auth resume \
  --state RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4 \
  --pkce oJ-rEXNu9YzqpCWVIPOwD5KvMhLAT73dstk0jye8nZ6 \
  https://localhost/?code=M.C521_BAY.2.U&state=RWdzST0ybUIzT1wtMSF9OCMmJHJUVmJrUmhhU0haLz4
```

### Refresh access token

```
$ ortie token refresh

Access token successfully refreshed (expires in 1h)
```

### Show access token

```
$ ortie token show

EwA4BOl3BAAUcDnR9grBJokeAHaUV8R3+rVHX+IAAQfw9oZLztQS8bo8NvyWmbs…
```

The `--auto-refresh` argument (as well as the config option `auto-refresh = true`) automatically refreshes expired tokens.

You can also inspect token metadata:

```
$ ortie token inspect

Token type: bearer
Issued: 22h 51m 1s ago
Expires in: 52m 38s
With refresh token: true
With scope: https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send
```

## Installation

The project is still experimental, it has not been released yet.

### Pre-built binary

Ortie CLI can be installed with a pre-built binary. Find the latest [releases](https://github.com/pimalaya/ortie/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. You should find a pre-built binary matching your OS.

*MacOS aarch64 and Windows i686 builds are broken, please use the next installation method until the first release.*

### Cargo (git)

Ortie CLI can also be installed with [cargo](https://doc.rust-lang.org/cargo/):

```
cargo install --locked --git https://github.com/pimalaya/ortie.git
```

## Configuration

The wizard is not yet available (it should come soon), so the only way to configure Ortie CLI is to copy the [sample config file](https://github.com/pimalaya/ortie/blob/master/config.sample.toml), to store it either at `~/.config/ortie.toml` or `~/.ortierc` then to customize it by commenting or uncommenting the options you need.

You will also need a registered application. This depends on your OAuth 2.0 provider. You can either use an existing application (public registration like Thunderbird) or register your own application. The first option is definitely simpler.

*See public Thunderbird application credentials for various providers at [github.com/mozilla](https://github.com/mozilla/releases-comm-central/blob/master/mailnews/base/src/OAuth2Providers.sys.mjs).*

### Google

```toml
endpoints.authorization = "https://accounts.google.com/o/oauth2/auth"
endpoints.token = "https://www.googleapis.com/oauth2/v3/token"
```

Using public Thunderbird application:

```toml
client-id = "406964657835-aq8lmia8j95dhl1a2bvharmfk3t1hgqj.apps.googleusercontent.com"
client-secret.raw = "kSmqreRr0qwBWJgbf5Y-PjSU"
```

Using your [own application](https://developers.google.com/identity/protocols/oauth2):

```toml
client-id = "<your-client-id>"
client-secret = "<your-client-secret>"
```

### Microsoft

```toml
endpoints.authorization = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
endpoints.token = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
```

Using public Thunderbird application:

```toml
client-id = "9e5f94bc-e8a4-4e73-b8be-63364c29d753"
endpoints.redirection = "https://localhost"
```

Using your [own application](https://learn.microsoft.com/en-us/exchange/client-developer/legacy-protocols/how-to-authenticate-an-imap-pop-smtp-application-by-using-oauth):

```toml
client-id = "<your-client-id>"
client-secret = "<your-client-secret>"
```

## FAQ

## Alternatives

- [pizauth](https://github.com/ltratt/pizauth): daemon-oriented alternative
- [oama](https://github.com/pdobsan/oama): haskell alternative
- [mutt_oauth2.py](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/mutt_oauth2.py): python script alternative

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that helped the project to receive financial support from various programs:

- [NGI Assure](https://nlnet.nl/project/Ortie/) in 2022
- [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/) in 2023
- [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/) in 2024 *(still ongoing)*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
