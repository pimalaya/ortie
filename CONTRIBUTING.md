# Contributing guide

Thank you for investing your time in contributing to Ortie.

## Development environment

The development environment is managed by [Nix flakes](https://nixos.wiki/wiki/Flakes). Running `nix develop` (or `nix-shell` for non-flake users) spawns a shell with the right Rust toolchain, `cargo-deny`, `pkg-config` and the OpenSSL / DBus libraries.

If you do not want to use Nix, install [rustup](https://rust-lang.github.io/rustup/index.html) and pull the toolchain pinned by `rust-version` in `Cargo.toml`:

```
rustup update
```

- `cargo` (>= `v1.88`)
- `rustc` (>= `v1.88`, edition 2024)

## Build

```
cargo build
```

You can disable default [features](https://doc.rust-lang.org/cargo/reference/features.html) with `--no-default-features` and enable individual features with `--features feat1,feat2`.

For example, a library-only release build with PKCE:

```
cargo build --no-default-features --features pkce,client,rustls-ring --release
```

## Project layout

Ortie is split in three layers, all in this repository:

- `src/{authorization_code_grant, issue_access_token, refresh_access_token}`: low-level I/O-free OAuth 2.0 coroutines (RFC 6749 + RFC 7636).
- `src/client.rs`: mid-level `OauthClient` that wraps the coroutines over a [pimalaya-stream](https://github.com/pimalaya/stream) connection (`client` feature).
- `src/cli/`: high-level CLI (`cli` feature, default). Flat layout:
  - `cli.rs`: root clap parser (`Cli`, `Command`).
  - `config.rs`: TOML DTO layer; all types end in `*Config` (`Config`, `AccountConfig`, `EndpointsConfig`, `StoragesConfig`, `StorageConfig`, `HooksConfig`, `HookStatusConfig`, `HookConfig`, `NotifyConfig`) and mirror the nested `[accounts.<name>]` shape.
  - `account.rs`: flat runtime `Account` built via `From<AccountConfig>`; carries the driver methods (`read_from_storage`, `write_to_storage`, `execute_on_{issue,refresh}_{success,error}_hook`, `redirection`).
  - `auth.rs`, `auth_get.rs`, `auth_resume.rs`: `AuthCommand` router and its `AuthGetCommand` / `AuthResumeCommand` leaves.
  - `token.rs`, `token_show.rs`, `token_refresh.rs`, `token_inspect.rs`: `TokenCommand` router and its `TokenShowCommand` / `TokenRefreshCommand` / `TokenInspectCommand` leaves.

The Pimalaya companion crates used at runtime are:

- [pimalaya/cli](https://github.com/pimalaya/cli): cross-binary CLI helpers (clap args, printer, spinner, build-time env, log filtering).
- [pimalaya/config](https://github.com/pimalaya/config): TOML loader, secret resolution, `std::process::Command` de/ser.
- [pimalaya/stream](https://github.com/pimalaya/stream): TCP / TLS plumbing shared by the std clients.
- [io-http](https://github.com/pimalaya/io-http): I/O-free HTTP/1.1 send coroutine.

## Lint, test, audit

```
cargo fmt
cargo clippy --all-features --all-targets
cargo test --all-features
cargo deny check
```

## Commit style

Ortie follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Prefix every commit with one of `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `ci`, `build`, optionally scoped (`fix(client): …`).
