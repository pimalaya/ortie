# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/pimalaya/ortie/compare/v1.0.0...master
[1.0.0]: https://github.com/pimalaya/ortie/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/pimalaya/ortie/compare/root...v0.1.0
