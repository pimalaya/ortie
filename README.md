# 🔑 Ortie [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage OAuth tokens

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

- **Ready-made providers**: Google, Microsoft and Fastmail sections in the sample, the first two on Pimalaya's own applications.
- **Every common grant**: authorization code, device code, and client credentials by secret or signed JWT.
- **Token lifecycle**: refresh on demand or when the token is read, inspect, and revoke at the server.
- **Your own storage**: tokens read and written through your commands, such as `pass` or `secret-tool`.
- **Persistent session**: `ortie repl` unlocks the secret store once, then answers token commands over stdin.
- **MCP server**: an agent drives the grants through `ortie mcp`, and no tool ever returns a token.
- **JSON output**: `--json` on every data command, its schema given by `ortie json-schema`.
- **Full client**: TLS via Rustls (`rustls-ring` default, `rustls-aws`) or Native TLS (`native-tls`).

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

Ortie follows the OAuth 2.1 draft and its security best current practice (RFC 9700):

- Grants: no implicit nor resource owner password grant, both removed by OAuth 2.1.
- PKCE: S256 on every authorization code flow, `pkce = "plain"` or `false` for servers that need it.
- Redirects: sent as configured, a port-less loopback such as the default `http://127.0.0.1` getting a random port (RFC 8252).
- Refresh rotation: a refreshed token is stored as soon as it arrives, the previous refresh token kept when none rotates.

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

A configuration is loaded from the first valid path among:

- `$XDG_CONFIG_HOME/ortie/config.toml`
- `$HOME/.config/ortie/config.toml`
- `$HOME/.ortierc`

Override the path with `-c <PATH>`. Repeat it to merge several files, such as `-c public.toml -c private.toml`: the first one is the base and the next ones are deep-merged on top. [config.sample.toml](./config.sample.toml) lists the fields every provider shares with their values, then one section per provider, yours to uncomment: Google, Microsoft, Fastmail, and a custom one for an application of your own. Releases and packages ship it as share/doc/ortie/config.sample.toml, which the welcome of a first run copies from.

## Usage

Every command and flag is documented behind `--help`. A complete flow, from the sample to a token:

```sh
curl -sSL --create-dirs -o ~/.config/ortie/config.toml https://raw.githubusercontent.com/pimalaya/ortie/master/config.sample.toml
$EDITOR ~/.config/ortie/config.toml # uncomment your provider, set the storage commands
ortie auth get                      # sign in, then store the first token
ortie token show                    # print the access token, refreshed once expired
ortie token inspect                 # print its type, scopes and expiry
ortie token revoke                  # revoke it at the server
```

When the redirection cannot be caught, `auth get` prints the `auth listen` or `auth resume` command finishing the flow.

### REPL

`ortie repl` holds one account in memory and answers `auth` and `token` commands, one per line, until `exit` or EOF. The secret store is read once for the whole session.

Interactively, it chains commands without unlocking the store each time:

```
$ ortie repl
ortie> auth get
ortie> token show
ortie> token inspect
ortie> exit
```

Non-interactively, a program spawns `ortie repl --json` and talks to it over stdio: it writes one command per line on stdin and reads one JSON value per line on stdout, failures included. A mail client can keep it running and write `token show` whenever it needs a fresh token.

### MCP

`ortie mcp` also talks over stdio, but speaks the [Model Context Protocol](https://modelcontextprotocol.io) so an agent can drive the grants. The agent starts one with `auth_get` and follows it with `auth_status`, while Ortie keeps the state, the PKCE verifier and the device code. No tool returns the token, and `token_revoke` needs `--allow-revoke`.

MCP clients start the servers declared in their configuration, most of them under an `mcpServers` object:

```json
{
  "mcpServers": {
    "ortie": {
      "command": "ortie",
      "args": ["mcp"]
    }
  }
}
```

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
