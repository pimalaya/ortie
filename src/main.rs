//! # Ortie
//!
//! A CLI managing OAuth 2.0 tokens from a TOML configuration, and the
//! architecture document of the repository: how the binary is layered
//! and where each concern lives. The roadmap lives in cairn/changes/,
//! the current truth in cairn/spec/, the landed history in cairn/log/.
//!
//! ## Layering
//!
//! Ortie is a thin, config-driven front-end over [io-oauth], the OAuth
//! engine: I/O-free coroutines organised per RFC plus the std-blocking
//! Oauth20ClientStd pump. This repository is only the glue around it.
//!
//! Parsing starts in [`cli`], the root clap parser. Bare `ortie` shows
//! the help; otherwise it routes into [`auth`], which obtains tokens by
//! running the account's grant, and [`token`], which works on the stored
//! one.
//!
//! [`repl`] holds those same two trees open against one account, so the
//! secret store is unlocked once instead of per command.
//!
//! [`mcp`] serves them to an agent as Model Context Protocol tools, the
//! flows running in the server so no secret reaches the agent.
//!
//! The config is user-owned: Ortie never writes it, and an account
//! starts from one of the samples/ files, config.sample.toml documenting
//! every field.
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
//! Each command prints one `*Output` type, a confirmation included, and
//! [`json_schema`] maps its invocation to that type's JSON Schema, which
//! `ortie json-schema` writes out.
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

mod account;
mod auth;
mod cli;
mod config;
mod json_schema;
mod mcp;
mod repl;
mod token;

use anyhow::Error;
use clap::{CommandFactory, Parser};
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};
use simd_cpu::SimdCpu;

use crate::cli::Cli;

fn main() {
    // NOTE: the base64 codecs below read the installed CPU token, the
    // portable baseline until one is installed.
    let _ = SimdCpu::install_detected();

    let cli = Cli::parse();

    Logger::try_init(&cli.log).expect("init logger");
    let mut printer = StdoutPrinter::new(&cli.json);

    let config_paths = cli.config.paths.as_ref();
    let account_name = cli.account.name.as_deref();

    let result = match cli.cmd {
        Some(cmd) => cmd.execute(&mut printer, config_paths, account_name),
        None => Cli::command().print_help().map_err(Error::from),
    };

    ErrorReport::eval(&mut printer, result)
}
