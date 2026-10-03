//! # Ortie
//!
//! A CLI managing OAuth 2.0 tokens from a TOML configuration, and the
//! architecture document of the repository: how the binary is layered
//! and where each concern lives. The roadmap lives in cairn/changes/,
//! the current truth in cairn/spec/, the landed history in cairn/log/.
//!
//! ## Layering
//!
//! Ortie is a thin, config-driven front-end. The OAuth engine, I/O-free
//! coroutines organised per RFC plus the std-blocking Oauth20ClientStd
//! pump, lives in [io-oauth]; PIM service discovery lives in
//! [io-pim-discovery]. This repository is only the glue between the two.
//!
//! Parsing starts in [`cli`], the root clap parser. Bare `ortie` runs
//! the configuration wizard, the natural first contact with the tool;
//! otherwise it routes into [`auth`], which obtains tokens by running
//! the account's grant, and [`token`], which works on the stored one.
//!
//! [`repl`] holds those same two trees open against one account, so the
//! secret store is unlocked once instead of per command.
//!
//! The [`wizard`] ends on a bare, valid TOML fragment: written to a
//! config file, appended to the one already there, or printed on stdout,
//! which is where `--json` and a redirected stdout stop. Its banner,
//! prompts and spinners render on stderr.
//!
//! An existing config is appended to, never rewritten and never
//! unconfirmed, so the config stays user-owned: there is no account
//! management command tree and none is planned. The wizard configures
//! only what it discovers, and `auth get` authorizes what it produced.
//!
//! Configuration is two layers. [`config`] holds the pure TOML DTOs,
//! each type ending in `*Config` and mirroring the nested
//! `[accounts.<name>]` shape. [`account`] flattens the account `-a` or
//! `default = true` selects into the runtime view commands consume.
//!
//! ## Conventions
//!
//! Endpoints are optional at parse time: each command checks the ones
//! it needs and fails naming the missing field. `token show` therefore
//! works on an account holding only a client id and the storage
//! commands, while `auth get` requires its grant's endpoints.
//!
//! Ortie never persists tokens itself: reads and writes go through
//! user-configured shell commands (pass, secret-tool, ...), and hooks
//! fire on issuance and refresh with the outcome exposed as environment
//! variables. Secrets travel as SecretString and are never logged.
//!
//! Everything the user asked for goes to stdout, data and errors alike
//! (JSON with `--json`), distinguished only by the exit code; stderr
//! carries the logs. Doc comments on the command structs double as the
//! CLI help: the first paragraph is `-h`, the rest completes `--help`.
//!
//! Each command emitting data prints one `*Output` type, and
//! [`json_schema`] maps its invocation to that type's JSON Schema, which
//! `ortie json-schema` writes out. A command reporting a confirmation
//! prints a `Message` and carries no schema.
//!
//! Device authorization (RFC 8628) is `grant = "device"`. The headless
//! grants are `grant = "client-credentials"` (RFC 6749 section 4.4),
//! authenticated by the client secret, and `grant =
//! "client-credentials-jwt"` (RFC 7523 section 2.2).
//!
//! The latter signs a JWT assertion with `client-key` and carries the
//! `x5t` thumbprint of `client-certificate`. Neither issues a refresh
//! token, so auto-refresh silently re-runs the grant instead of
//! exchanging one.
//!
//! [io-oauth]: https://docs.rs/io-oauth
//! [io-pim-discovery]: https://docs.rs/io-pim-discovery

mod account;
mod auth;
mod cli;
mod config;
mod json_schema;
mod repl;
mod token;
#[cfg(feature = "wizard")]
mod wizard;

use std::{
    io::{IsTerminal, stdin},
    path::PathBuf,
};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::Printer, printer::StdoutPrinter};
use pimalaya_config::toml::TomlConfig;

use crate::{cli::Cli, config::Config};

fn main() {
    let cli = Cli::parse();

    Logger::try_init(&cli.log).expect("init logger");
    let mut printer = StdoutPrinter::new(&cli.json);

    let config_paths = cli.config.paths.as_ref();
    let account_name = cli.account.name.as_deref();

    let result = match cli.cmd {
        Some(cmd) => cmd.execute(&mut printer, config_paths, account_name),
        None => meet_bare_invocation(&mut printer, config_paths, account_name.is_some()),
    };

    ErrorReport::eval(&mut printer, result)
}

/// Meets a bare `ortie`, which is where a newcomer lands.
///
/// A missing configuration raises the offer; anything else gets the
/// help, scripts and JSON callers included. A broken file counts as a
/// configuration, and `--account` alone as a half-typed command.
fn meet_bare_invocation(
    printer: &mut StdoutPrinter,
    config_paths: &[PathBuf],
    named_account: bool,
) -> Result<()> {
    let configured = Config::from_paths_or_default(config_paths)
        .ok()
        .flatten()
        .is_some();

    if !configured && !named_account && !printer.is_json() && stdin().is_terminal() {
        let path = Config::target_path(config_paths)?;

        // NOTE: a bare invocation has nothing to run after the offer, so
        // a declined one falls back to the help.
        if cli::offer_configuration(printer, config_paths, &path)? {
            return Ok(());
        }
    }

    Cli::command().print_help()?;

    Ok(())
}
