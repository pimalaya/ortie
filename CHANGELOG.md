# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Merged the io-oauth library into this repo. The crate now exposes low-level I/O-free OAuth 2.0 coroutines (always on, gated only by `pkce` for PKCE support), a mid-level std blocking client (`client` feature, `OauthClient`), and the existing CLI (`cli` feature, default).
- Added a mid-level `OauthClient` wrapper that drives the v2_0 coroutines over a `pimalaya-stream` connection. Both the auth resume and token refresh CLI paths route through it.

### Changed

- Re-licensed the project from AGPL-3.0-only to dual MIT OR Apache-2.0 to align with the merged library.
- Migrated from pimalaya-toolbox to the split pimalaya-cli / pimalaya-config / pimalaya-stream stack.
- Reorganised the source tree. Flattened the former `v2_0/` module to the crate root (`ortie::{authorization_code_grant, issue_access_token, refresh_access_token}`) since OAuth 2.0 is the only protocol version supported. Moved every CLI-feature-gated module under a single flat `cli/` folder: the root clap parser (`cli/cli.rs` exposing `Cli` + `Command`), the nested TOML DTO layer (`cli/config.rs` exposing `Config`, `AccountConfig`, `EndpointsConfig`, `StoragesConfig`, `StorageConfig`, `HooksConfig`, `HookStatusConfig`, `HookConfig`, `NotifyConfig`), the flat runtime view built from the DTO via `From<AccountConfig>` (`cli/account.rs` exposing `Account` with driver methods `read_from_storage`, `write_to_storage`, `execute_on_{issue,refresh}_{success,error}_hook`, `redirection`), and one file per subcommand router / leaf (`cli/auth.rs` + `cli/auth_get.rs` + `cli/auth_resume.rs`, `cli/token.rs` + `cli/token_show.rs` + `cli/token_refresh.rs` + `cli/token_inspect.rs`). `client.rs` stays at the crate root. The binary entry point imports `ortie::cli::Cli`.
- Replaced the deprecated `--debug` and `--trace` CLI flags with `--log-level <level>` and `--log-file <path>` (provided by pimalaya-cli).
- Updated to the new io-http HTTP/1.1 send coroutine (`rfc9112::send::Http11Send`).
- Made OAuth 2.0 the always-on baseline. Dropped the `oauth2` cargo feature (and the `rfc6749` alias); the `v2_0` module is now part of the lib unconditionally.
- Replaced the deprecated `io-process` dependency with `std::process::Command` and the de/serializer helpers from `pimalaya-config::command`. Storage and hook commands now accept the standard TOML shapes (string wrapped through `sh -c`, or `[program, arg, ...]` array); the previous `expand = true` behaviour is folded into the string form. List-form commands no longer expand environment variables in argv, use the string form when you want shell substitution.

### Removed

- Dropped the `oauth2` cargo feature (now always on) and its `rfc6749` alias.
- Dropped the `command` cargo feature; shell-command storage and hooks are now part of the CLI baseline.
- Dropped the `io-process` dependency. Configurations that relied on env-var expansion inside list-form commands must switch to the string form to keep that behaviour.

## [1.1.0] - 2026-02-16

### Changed

- Replaced default TLS feature `native-tls` by `rustls-ring`. Native TLS makes release process a bit more complicated and heavier static binaries. Rustls + Ring seems to be a better choice.

### Removed

- Removed direct keyring support. Use commands instead. The reason is that keyring support has always been a bit blurry for users. It's hard to know what it truly does behind the scene. Plus it increases the complexity. The same way Ortie CLI exports OAuth logic and simplies usage inside tools, [Mimosa CLI](https://github.com/pimalaya/mimosa) does the same for passwords and keyring.

## [1.0.0] - 2026-02-12

### Added

- Added support for custom authorization parameters ([#4]).

### Changed

- Changed default cargo features to `native-tls`, `command`, `keyring` and `notify`.
- Made the redirection endpoint optional. If omitted, `http://127.0.0.1:0` is used by default, which will start the redirection server on a random port ([#3]).
- Replaced `on-issue-access-token` by `hooks.on-issue`.
- Replaced `on-refresh-access-token` by `hooks.on-refresh`.

### Fixed

- Fixed release build with `native-tls` and `keyring` features.

## [0.1.0] - 2025-10-24

### Changed

- Init auth and token commands
- Replaced pimalaya tui by toolbox
- Bumped all dependencies

### Fixed

- Fix CI and release builds

[#3]: https://github.com/pimalaya/ortie/issues/3
[#4]: https://github.com/pimalaya/ortie/issues/4

[unreleased]: https://github.com/pimalaya/ortie/compare/v1.1.0...master
[1.1.0]: https://github.com/pimalaya/ortie/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/pimalaya/ortie/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/pimalaya/ortie/compare/root...v0.1.0
