//! # Repl command
//!
//! The `repl` subcommand, a session over stdin and stdout holding one
//! account open, so the secret store is read once instead of per run.
//!
//! One-shot commands re-read it every time, and a keyring confirming
//! disclosure per process prompts again and again. Holding the token in
//! memory collapses those prompts into a single unlock, plus one per
//! refresh write, which must persist a rotated refresh token.
//!
//! The session is bound to the `-a` or default account like any other
//! command, and each input line is parsed as a `token` or `auth`
//! subcommand dispatched against the in-memory account. pimalaya-cli's
//! [`Repl`] runs the loop: lines split as a shell splits them, the
//! session flags refused, every answer on stdout.
//!
//! The stdio pipe is the authorisation boundary, so the token is never
//! handed to another process the way a shared agent would. On exit the
//! account drops and its secrets are zeroized.

use anyhow::Result;
use clap::{Parser, Subcommand};
use pimalaya_cli::{error::NoErrorCode, printer::Printer, repl::Repl};

use crate::{
    account::Account,
    auth::{get::AuthGetCommand, listen::AuthListenCommand, resume::AuthResumeCommand},
    token::TokenCommand,
};

/// Start a persistent REPL session for one account.
///
/// The account token is read from storage on first use and held in
/// memory for the session, so the keyring is unlocked once instead of
/// on every command. Each input line is a `token` or `auth` subcommand
/// (`token show`, `auth get`); `quit`, or EOF, ends the session.
#[derive(Debug, Parser)]
pub struct ReplCommand;

/// The `token` and `auth` commands, the subset of the CLI tree running
/// against the in-memory account.
#[derive(Debug, Subcommand)]
enum ReplCommandTree {
    #[command(subcommand)]
    Token(TokenCommand),
    #[command(subcommand)]
    Auth(ReplAuthCommand),
}

impl ReplCommandTree {
    /// Dispatches the parsed line against the in-memory account.
    fn execute(
        self,
        printer: &mut impl Printer,
        account: &mut Account,
        invocation: &str,
    ) -> Result<()> {
        match self {
            Self::Token(cmd) => cmd.execute(printer, account),
            Self::Auth(cmd) => cmd.execute(printer, account, invocation),
        }
    }
}

/// Get a fresh access token by running the account's OAuth grant.
///
/// The token it issues is the one the next `token` commands of the session
/// read.
#[derive(Debug, Subcommand)]
enum ReplAuthCommand {
    Get(AuthGetCommand),
    Listen(AuthListenCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl ReplAuthCommand {
    fn execute(
        self,
        printer: &mut impl Printer,
        account: &mut Account,
        invocation: &str,
    ) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account, invocation),
            Self::Listen(cmd) => cmd.execute(printer, account),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}

impl ReplCommand {
    /// Runs the read-eval-print loop until EOF or a quit command,
    /// keeping the resolved token in the account across iterations,
    /// each line printing as `printer` does unless it asks for JSON.
    ///
    /// The `invocation` is handed to `auth get`, for the `auth listen` and
    /// `auth resume` commands it prints when the flow hands off.
    pub fn execute(
        self,
        printer: &impl Printer,
        mut account: Account,
        invocation: &str,
    ) -> Result<()> {
        let repl = Repl::new("ortie", printer.is_json());

        repl.run::<ReplCommandTree, NoErrorCode>(|printer, cmd| {
            cmd.execute(printer, &mut account, invocation)
        })
    }
}
