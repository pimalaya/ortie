# Contributing guide

Thank you for investing your time in contributing to Ortie.

## Development environment

The development environment is managed by [Nix flakes](https://wiki.nixos.org/wiki/Flakes). Running `nix develop` (or `nix-shell` for non-flake users) spawns a shell with the right Rust toolchain, `cargo-deny`, `pkg-config` and the OpenSSL / DBus libraries.

If you do not want to use Nix, install [rustup](https://rust-lang.github.io/rustup/index.html) and pull the `cargo` / `rustc` toolchain pinned by `rust-version` in Cargo.toml (edition 2024):

```sh
rustup update
```

## Build

```sh
cargo build
```

You can disable default [features](https://doc.rust-lang.org/cargo/reference/features.html) with `--no-default-features` and enable individual features with `--features feat1,feat2`. For example, a release build with native TLS instead of the default Rustls:

```sh
cargo build --no-default-features --features native-tls --release
```

## Project layout

Ortie is a pure CLI binary: the OAuth engine (I/O-free coroutines organised per RFC, plus the std-blocking `Oauth20ClientStd` pump) lives in [io-oauth](https://github.com/pimalaya/io-oauth), and PIM service discovery (consumed by the auth discover wizard) lives in [io-pim-discovery](https://github.com/pimalaya/io-pim-discovery). This repository only contains the glue between the user's TOML config and those two crates.

The entry point src/main.rs doubles as the architecture document, the same way lib.rs does for io-oauth: read its header first. In short, src/cli.rs declares the root clap parser, src/config.rs holds the TOML DTO layer (every type ends in `*Config` and mirrors the nested `[accounts.<name>]` shape), src/account.rs flattens the selected account into the runtime `Account` view carrying the storage and hook drivers, and the two command trees live under src/auth (discover, get, resume) and src/token (show, inspect, refresh).

Doc comments on the command structs double as the CLI help: the first paragraph (two lines at most) is what `-h` shows, the following paragraphs complete the `--help` page. Keep them present and tight on every pub item.

At runtime Ortie also relies on the cross-binary Pimalaya helpers: [pimalaya/cli](https://github.com/pimalaya/cli) for clap args, printer and prompt primitives, [pimalaya/config](https://github.com/pimalaya/config) for the TOML loader, secret resolution and shell-command de/serialization, and [pimalaya/stream](https://github.com/pimalaya/stream) for the TCP / TLS plumbing. Bugs touching OAuth wire semantics belong in io-oauth and discovery bugs in io-pim-discovery; config shape, storage, hooks and command UX live here.

## Override dependencies

Ortie builds against the published Pimalaya crates. When hacking on a companion crate, patch it to your local checkout in Cargo.toml:

```toml
[patch.crates-io]
io-oauth.path = "../io-oauth"
```

If cargo complains about *"perhaps two different versions of crate X are being used"*, patch every Pimalaya crate that pulls X transitively so the dep graph converges on the local copies.

## Lint, test, audit

```sh
cargo fmt
cargo clippy --all-features --all-targets
cargo test --all-features
cargo deny check
```

## Commit style

Ortie follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Prefix every commit with one of `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `ci`, `build`, optionally scoped (`fix(auth): …`).
