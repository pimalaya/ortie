//! `repl` subcommand: a persistent session over stdin/stdout.
//!
//! One-shot commands re-read the secret store on every run, so a
//! keyring that confirms disclosure per process prompts the user again
//! and again. The REPL resolves the account token once and holds it in
//! memory for the life of the process, collapsing those prompts into a
//! single unlock (plus one per refresh write, which must persist a
//! rotated refresh token).
//!
//! The session is bound to one account (the `-a` or default one, like
//! any other command) and reuses the `token` command grammar: each
//! input line is parsed as a `token` subcommand and dispatched against
//! the in-memory account, which carries the resolved token across
//! iterations. The persistence is private to whoever spawned the
//! process (the stdio pipe is the authorisation boundary), so it does
//! not hand the token to any other process the way a shared agent
//! would. On exit the account drops and its secrets are zeroized.

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
/// The account token is read from storage once, on first use, and held
/// in memory for the session, so the keyring is unlocked a single time
/// instead of on every command. Each input line is a `token` or `auth`
/// subcommand (e.g. `token show`, `auth get`); type `quit` (or send EOF)
/// to end.
#[derive(Debug, Parser)]
pub struct ReplCommand;

/// One parsed REPL input line: the same command grammar as the CLI
/// (minus the binary name), scoped to the account-bound commands.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct ReplLine {
    #[command(subcommand)]
    cmd: ReplCommandTree,
}

/// The subset of the CLI command tree available inside the REPL: the
/// `token` and `auth` commands, which run against the in-memory
/// account. The account-less `auth discover` wizard has no REPL form
/// (bare `ortie` runs it).
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

/// The account-scoped `auth` leaves usable inside the REPL: `get` and
/// `resume`, dispatched against the session's account so a token they
/// issue is immediately visible to the following `token` commands.
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

            // Commands write their result to stdout without a trailing
            // newline (so one-shot output pipes cleanly); in the loop we
            // terminate and flush each result, otherwise the line-buffered
            // stdout holds it until the session ends. A parse or command
            // error is reported and the loop continues; a bad line never
            // terminates the session.
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
