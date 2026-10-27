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
//! subcommand dispatched against the in-memory account.
//!
//! The stdio pipe is the authorisation boundary, so the token is never
//! handed to another process the way a shared agent would. On exit the
//! account drops and its secrets are zeroized.

use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};
use pimalaya_cli::{error::ErrorReport, printer::Printer};

use crate::{
    account::Account,
    auth::{get::AuthGetCommand, resume::AuthResumeCommand},
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

/// One parsed REPL input line, the CLI grammar minus the binary name,
/// scoped to the account-bound commands.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct ReplLine {
    #[command(subcommand)]
    cmd: ReplCommandTree,
}

/// The `token` and `auth` commands, the subset of the CLI tree running
/// against the in-memory account.
///
/// The configuration wizard has no REPL form, being account-less: bare
/// `ortie` is what runs it.
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
        resume: &str,
    ) -> Result<()> {
        match self {
            Self::Token(cmd) => cmd.execute(printer, account),
            Self::Auth(cmd) => cmd.execute(printer, account, resume),
        }
    }
}

/// The `auth` leaves usable inside the REPL.
///
/// They dispatch against the session's account, so a token they issue
/// is immediately visible to the `token` commands that follow.
#[derive(Debug, Subcommand)]
enum ReplAuthCommand {
    Get(AuthGetCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl ReplAuthCommand {
    fn execute(
        self,
        printer: &mut impl Printer,
        account: &mut Account,
        resume: &str,
    ) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account, resume),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}

impl ReplCommand {
    /// Runs the read-eval-print loop until EOF or a quit command,
    /// keeping the resolved token in the account across iterations.
    ///
    /// The `resume` invocation is handed to `auth get`, for the
    /// `auth resume` command it prints when the flow hands off.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        mut account: Account,
        resume: &str,
    ) -> Result<()> {
        let stdin = io::stdin();
        let interactive = stdin.is_terminal();

        let mut line = String::new();
        loop {
            if interactive {
                eprint!("\nortie> ");
                let _ = io::stderr().flush();
            }

            line.clear();
            if stdin.read_line(&mut line)? == 0 {
                break; // EOF
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }
            if matches!(input, "quit" | "exit") {
                break;
            }

            let result = match ReplLine::try_parse_from(input.split_whitespace()) {
                Ok(ReplLine { cmd }) => cmd.execute(printer, &mut account, resume),
                // NOTE: help and version are not failures, and clap
                // prints them on stdout already.
                Err(err) if !err.use_stderr() => {
                    let _ = err.print();
                    continue;
                }
                Err(err) if interactive => {
                    let _ = err.print();
                    continue;
                }
                Err(err) => Err(err.into()),
            };

            match result {
                // NOTE: the printer ends each JSON value with a newline,
                // flushed when piped, while a text result carries no
                // trailing newline so one-shot output pipes cleanly: the
                // loop terminates and flushes it, or the line-buffered
                // stdout holds it until the session ends.
                Ok(()) => {
                    if !printer.is_json() {
                        println!();
                        let _ = io::stdout().flush();
                    }
                }
                Err(err) if interactive => eprintln!("{err:?}"),
                // NOTE: a piped session is a machine channel, so its
                // errors reach stdout through the printer like any other
                // command's, one per line, and stderr keeps the logs.
                Err(err) => {
                    let report: ErrorReport = err.into();
                    printer.out(report)?;
                    io::stdout().flush()?;
                }
            }
        }

        Ok(())
    }
}
