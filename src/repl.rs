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
use pimalaya_cli::printer::Printer;

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
    fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        match self {
            Self::Token(cmd) => cmd.execute(printer, account),
            Self::Auth(cmd) => cmd.execute(printer, account),
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
    fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}

impl ReplCommand {
    /// Runs the read-eval-print loop until EOF or a quit command,
    /// keeping the resolved token in the account across iterations.
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
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

            // NOTE: commands write their result with no trailing
            // newline, so one-shot output pipes cleanly; the loop has to
            // terminate and flush each one, or the line-buffered stdout
            // holds it until the session ends.
            match ReplLine::try_parse_from(input.split_whitespace()) {
                Ok(ReplLine { cmd }) => match cmd.execute(printer, &mut account) {
                    Ok(()) => {
                        println!();
                        let _ = io::stdout().flush();
                    }
                    Err(err) => eprintln!("{err:?}"),
                },
                Err(err) => {
                    let _ = err.print();
                }
            }
        }

        Ok(())
    }
}
