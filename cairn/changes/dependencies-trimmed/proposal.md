---
cairn: change
id: dependencies-trimmed
status: landed
created: 2026-11-09
---

# Dependencies trimmed

## Why

samples-replace-the-wizard (landed 2026-11-08) took the wizard and its terminal handling out; the command-line interfaces are moving to plain text and JSON output, with no colour and no terminal queries. What is left was measured on 2026-11-09, each candidate built alone from 2913ba0 (release profile, fat LTO, stripped; 4,520,680 bytes, 141 crates in the normal tree):

- url's IDNA back end, ICU4X: 133,512 bytes and 21 crates, for internationalised host names no OAuth endpoint uses.
- open: 22,240 bytes and 3 crates (open, is-wsl, is-docker), to launch a browser `auth get` already prints the link for. Over SSH or in a container the attempt only adds a failure line, and the `--json` hand-off and the MCP server never open anything.
- Colour and terminal width: about 22 KB and 10 crates (anstream and its parts, terminal_size, rustix, linux-raw-sys, bitflags), through clap's `color` and `wrap_help` and pimalaya-cli's env_logger `auto-color`.
- notify-rust: nothing in the default build, the `notify` feature being off, but four hook keys under `cfg`, a libdbus link with its package.nix branch, a `vendored` forward, and the active hook-variables-camel-case change, which would move it onto pimalaya-config's notification template. A hook command running `notify-send` or `osascript` does the same.
- The JWT client credentials grant (io-oauth `jwt-bearer`): 294,480 bytes and 17 crates. Kept, see Decisions.

The dev-dependencies env_logger, rustls and rustls-platform-verifier are used by no test. rustls's default features compile aws-lc-rs into every test build, so the tests ran with a crypto backend the release does not ship.

## What

- **Every dependency takes `default-features = false`** and names the features Ortie uses. clap drops `color` and `wrap_help`.
- **url's IDNA runs without a Unicode back end**: `idna_adapter` pinned to 1.0.0 in Cargo.toml, so a `cargo install` without `--locked` keeps the pin too. A host is ASCII, an internationalised one written in its Punycode form (`xn--…`), and a non-ASCII host is refused when its URL is parsed.
- **open removed.** `auth get` prints the authorization URI, or the verification URI and user code, and opens nothing. The authorization text says to open the link; the device text names the complete verification URI when the server sends one.
- **The `notify` feature removed**, with the hooks' `notify` tables, `NotifyConfig`, notify-rust, the direct dirs and shellexpand dependencies (only the notification used them), the `vendored` forward and package.nix's dbus branch. A `notify` table is refused as an unknown key, as every build without the feature already did.
- **Dev-dependencies**: env_logger, rustls and rustls-platform-verifier removed; tempfile without `getrandom`.
- **cli-008** (pimalaya/.github GUIDELINES.md): a hook may show a `notify`, which a product leaves out when its events need no desktop notification.
- **hook-variables-camel-case** (active) shrinks to the variable renames.

## Decisions

Accepted by the owner on 2026-11-09 ("Go for the whole plan").

- **The JWT grant stays in the default build.** Shipped in 2.1.0 for Microsoft certificate credentials, its users run the distribution and Nix binaries a non-default feature would leave it out of. Its real cost is the RUSTSEC-2023-0071 ignore in deny.toml, not the bytes.
- **No Unicode back end rather than unicode-rs**: idna_adapter 1.1 builds 128,952 bytes larger than ICU4X.
- **A `notify` table is refused, not ignored**: the default build already refused it, and a hook that silently stops notifying is worse than a configuration error naming the key.

## Not in this change

- pimalaya-cli's `terminal` feature turning on env_logger's `auto-color` and clap's `wrap_help`: its own change in pimalaya-cli, logs-and-help-stay-plain (65d6d30, pushed 2026-11-09), which Ortie pins through `[patch.crates-io]` until pimalaya-cli's next release. clap wraps nothing without `wrap_help`, so help prints each paragraph on one line: the owner chose that over a fixed width, which would bring terminal_size and rustix back.
- io-oauth drawing its random bytes from getrandom rather than rand, and pimalaya-config leaving toml's writer out: their own repositories.
