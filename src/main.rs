//! # Ortie
//!
//! CLI to manage OAuth 2.0 tokens, configured through TOML. This header is the
//! architecture document of the repository: it explains how the binary is
//! layered and where each concern lives, the same way the io-oauth lib.rs does
//! for the engine.
//!
//! ## Layering
//!
//! Ortie is a thin, config-driven front-end. The OAuth engine itself
//! (I/O-free coroutines organised per RFC, plus the std-blocking
//! Oauth20ClientStd pump) lives in [io-oauth]; PIM service discovery
//! (consumed by the discovery wizard) lives in
//! [io-pim-discovery]. This repository only contains the CLI glue
//! between the user's config and those two crates.
//!
//! [io-oauth]: https://docs.rs/io-oauth
//! [io-pim-discovery]: https://docs.rs/io-pim-discovery
//!
//! Parsing starts in [`cli`], the root clap parser. Bare `ortie` (no
//! subcommand) runs the configuration wizard, the natural first
//! contact with the tool; otherwise it routes into two command trees:
//! [`auth`] obtains tokens by running the OAuth grant configured on
//! the account (get, resume), while [`token`] works on the
//! token already persisted in storage (show, inspect, refresh).
//!
//! The [`wizard`] ends on a bare, valid TOML fragment: appended to a
//! config file when it offers to (writing to a terminal), printed on
//! stdout otherwise, so `ortie >> <config>` still works as the
//! write-back. Its banner, prompts and spinners render on stderr. An
//! existing config is appended to, never rewritten, so the config
//! stays user-owned; there is no account management command tree and
//! none is planned. The wizard configures only what it can discover,
//! and runs no grant of its own: authorizing the account it produced
//! is what `auth get` is for.
//!
//! Configuration is a two-layer affair. [`config`] holds the pure
//! TOML DTOs: every type ends in `*Config`, mirrors the nested
//! `[accounts.<name>]` shape and carries no behaviour. [`account`]
//! flattens the account selected by `-a` (or `default = true`) into
//! the runtime [`account::Account`] view that commands consume, along
//! with the driver methods for storage and hooks.
//!
//! ## Conventions
//!
//! Endpoints are optional at parse time: each command checks the ones
//! it actually needs and fails with an error naming the missing
//! field. `token show` therefore works on a minimal account holding
//! only a client id and the storage commands, while `auth get`
//! requires the endpoints of the configured grant.
//!
//! Ortie never persists tokens itself: reads and writes go through
//! user-configured shell commands (pass, secret-tool, ...), and hooks
//! fire on token issuance and refresh with the outcome exposed as
//! environment variables. Secrets travel as SecretString and are
//! never logged.
//!
//! Everything the user asked for goes to stdout, data and errors
//! alike (JSON with `--json`), distinguished only by the exit code;
//! stderr carries logs. Doc comments on the command structs double as
//! the CLI help: the first paragraph (two lines at most) is the `-h`
//! summary, the following paragraphs complete the `--help` page.
//!
//! Device authorization (RFC 8628) is selected with `grant = "device"`.
//! The headless client credentials grants are selected with
//! `grant = "client-credentials"` (RFC 6749 section 4.4, secret) and
//! `grant = "client-credentials-jwt"` (RFC 7523 section 2.2, JWT
//! client assertion signed with `client-key`, `x5t` thumbprint from
//! `client-certificate`); they issue no refresh token, so auto-refresh
//! silently re-runs the grant instead of exchanging a refresh token.
//! The remaining roadmap (discovery upgrades, revocation) lives in
//! cairn/changes/; current truth is in cairn/spec/ and landed history
//! in cairn/log/ (see <https://github.com/pimalaya/cairn>).

mod account;
mod auth;
mod cli;
mod config;
mod repl;
mod token;
mod wizard;

use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();

    Logger::try_init(&cli.log).expect("init logger");
    let mut printer = StdoutPrinter::new(&cli.json);

    let result = match cli.cmd {
        Some(cmd) => {
            let config_paths = cli.config.paths.as_ref();
            let account_name = cli.account.name.as_deref();
            cmd.execute(&mut printer, config_paths, account_name)
        }
        None => wizard::run(&mut printer),
    };

    ErrorReport::eval(&mut printer, result)
}
