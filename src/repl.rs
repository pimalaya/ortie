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
//! subcommand dispatched against the in-memory account. Lines are split as
//! a shell splits them, so a quoted value keeps its spaces.
//!
//! The stdio pipe is the authorisation boundary, so the token is never
//! handed to another process the way a shared agent would. On exit the
//! account drops and its secrets are zeroized.

use std::io::{self, IsTerminal, Write};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use pimalaya_cli::{
    clap::args::JsonFlag,
    error::ErrorReport,
    printer::{Printer, StdoutPrinter},
};

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

/// Run a `token` or `auth` command against the session's account.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct ReplLine {
    #[command(subcommand)]
    cmd: ReplCommandTree,
    /// Switch this line's output to JSON, the session's other lines
    /// keeping theirs.
    #[command(flatten)]
    json: JsonFlag,
}

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
    /// keeping the resolved token in the account across iterations.
    ///
    /// The `invocation` is handed to `auth get`, for the `auth listen` and
    /// `auth resume` commands it prints when the flow hands off.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        mut account: Account,
        invocation: &str,
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

            let result = match shlex::split(input).map(ReplLine::try_parse_from) {
                None => Err(anyhow!("Cannot split the line: a quote is not closed")),
                // NOTE: a line asking for JSON in a text session prints
                // through a JSON printer of its own, its failure included.
                Some(Ok(ReplLine { cmd, json })) if json.enabled && !printer.is_json() => {
                    let mut printer = StdoutPrinter::new(&json);
                    let result = cmd.execute(&mut printer, &mut account, invocation);
                    answer(&mut printer, interactive, result)?;
                    continue;
                }
                Some(Ok(ReplLine { cmd, .. })) => cmd.execute(printer, &mut account, invocation),
                // NOTE: help and version are not failures, and clap
                // prints them on stdout already.
                Some(Err(err)) if !err.use_stderr() => {
                    let _ = err.print();
                    continue;
                }
                Some(Err(err)) if interactive => {
                    let _ = err.print();
                    continue;
                }
                Some(Err(err)) => Err(err.into()),
            };

            answer(printer, interactive, result)?;
        }

        Ok(())
    }
}

/// Prints the outcome of one line: the result is already out, ending with
/// its newline, and a failure goes to stderr when interactive and through
/// the printer on stdout when piped.
fn answer(printer: &mut impl Printer, interactive: bool, result: Result<()>) -> Result<()> {
    match result {
        Ok(()) => io::stdout().flush()?,
        Err(err) if interactive => eprintln!("{err:?}"),
        // NOTE: a piped session is a machine channel, so its errors reach
        // stdout through the printer like any other command's, one per
        // line, and stderr keeps the logs.
        Err(err) => {
            let report: ErrorReport = err.into();
            printer.out(report)?;
            io::stdout().flush()?;
        }
    }

    Ok(())
}
