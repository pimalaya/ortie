//! # Ortie
//!
//! CLI to manage OAuth tokens, configured through TOML. This header
//! is the architecture document of the repository: it explains how
//! the binary is layered and where each concern lives, the same way
//! the io-oauth lib.rs does for the engine.
//!
//! ## Layering
//!
//! Ortie is a thin, config-driven front-end. The OAuth engine itself
//! (I/O-free coroutines organised per RFC, plus the std-blocking
//! Oauth20ClientStd pump) lives in [io-oauth]; PIM service discovery
//! (consumed by the auth discover wizard) lives in
//! [io-pim-discovery]. This repository only contains the CLI glue
//! between the user's config and those two crates.
//!
//! [io-oauth]: https://docs.rs/io-oauth
//! [io-pim-discovery]: https://docs.rs/io-pim-discovery
//!
//! Parsing starts in [`cli`], the root clap parser. Bare `ortie` (no
//! subcommand) runs the discovery wizard, the natural first contact
//! with the tool; otherwise it routes into two command trees:
//! [`auth`] obtains tokens by running the OAuth grant configured on
//! the account (discover, get, resume), while [`token`] works on the
//! token already persisted in storage (show, inspect, refresh).
//!
//! The wizard never writes any file: it prints a complete, valid
//! TOML fragment on stdout with its guidance embedded as comments,
//! and prompts render on stderr, so `ortie >> <config>` is the
//! write-back. The config stays entirely user-owned; there is no
//! account management command tree and none is planned.
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
//! The roadmap (device grant, discovery upgrades, revocation) lives
//! in docs/oauth21-plan.md.

mod account;
mod auth;
mod cli;
mod config;
mod token;

use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::{auth::discover::AuthDiscoverCommand, cli::Cli};

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
        None => AuthDiscoverCommand { input: None }.execute(&mut printer),
    };

    ErrorReport::eval(&mut printer, result)
}
