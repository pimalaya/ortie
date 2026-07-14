# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the account discovery wizard, run by bare `ortie` (alias of `auth discover`).

  Prompts for an email address, a server or an issuer URI, discovers the reachable PIM services through [io-pim-discovery](https://github.com/pimalaya/io-pim-discovery) (a spinner reports the search progress), keeps the ones advertising an OAuth 2.0 method and prints the pick as a complete `[accounts.<name>]` fragment: valid TOML on stdout with guidance embedded as comments (prompts render on stderr, so `ortie >> <config>` appends it directly), or a JSON object with `--json`. The client step proposes well-known public applications registered against the discovered provider (Thunderbird for Google, Microsoft and Fastmail), each labelled with the PIM domains its registration covers and carrying its registered scopes (used when discovery yielded none); the custom entry prompts for client id and secret, scopes and redirection instead. The storage step plugs the token into a credential provider CLI known for the running OS (secret-tool, kwallet-query, security, pass), custom shell commands as fallback. The fragment enables auto-refresh, links config.sample.toml and lists the follow-up commands (auth get, auth resume, token show, token refresh). Ortie never writes the config itself.

- Added the `grant` account config field.

  Selects the OAuth 2.0 grant flow run by the auth commands; defaults to `authorization-code`, the previous implicit behavior.

- Added the `extras` account config table.

  Parameters forwarded verbatim to the authorization request query, for provider-specific knobs like Google's `access_type` / `prompt`, `login_hint`, or the RFC 8707 `resource` indicator; keys are wire parameter names, never kebab-case renamed.

- Added the token issuance time to the `token inspect` output.

  Read from the HTTP `Date` response header.

### Changed

- Turned ortie into a pure CLI binary over the extracted [io-oauth](https://github.com/pimalaya/io-oauth) crate.

  io-oauth owns the I/O-free OAuth 2.0 coroutines and the std blocking client; library users should depend on it directly. The source tree follows the command shape (src/auth, src/token) and the src/main.rs header doubles as the architecture document.

- Enabled PKCE by default with the S256 method, aligning with OAuth 2.1.

  The `pkce` config field accepts booleans (`true` = S256, `false` = off) and method strings (`"s256"`, `"plain"`); servers rejecting PKCE parameters need an explicit `pkce = false`.

- Made every `endpoints.*` config field optional at parse time.

  Each command checks the endpoints it actually needs, so `token show` works on a minimal account holding only `client-id` and the storage commands.

- Replaced the deprecated `--debug` and `--trace` CLI flags with `--log-level <level>` and `--log-file <path>`.

- Replaced the `io-process`-based storage and hook commands with `std::process::Command` and the `pimalaya-config::command` helpers.

  Commands accept the standard TOML shapes: a string wrapped through the platform shell (with env-var expansion) or an exec-style `[program, arg, ...]` array (no expansion).

- Re-licensed the project from AGPL-3.0-only to dual MIT OR Apache-2.0.

- Migrated from pimalaya-toolbox to the split pimalaya-cli / pimalaya-config / pimalaya-stream stack.

### Removed

- Removed the library target and every non-TLS cargo feature (`oauth2`, its `rfc6749` alias, `command`, `cli`, `client`).

  The binary always builds with OAuth, shell-command storage / hooks and the full CLI included; remaining features are the TLS providers (`rustls-ring` default, `rustls-aws`, `native-tls`), `vendored` and `notify`.

- Removed the `io-process` dependency.

  Configurations relying on env-var expansion inside list-form commands must switch to the string form.

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
