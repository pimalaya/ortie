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
//! command, and each input line is any command of the binary, run over
//! the [`Session`] holding that account. pimalaya-cli's [`Repl`] runs the
//! loop: lines split as a shell splits them, the session flags refused,
//! a nested `repl` or `mcp` line refused coded `usage-invalid` before it
//! reaches this crate, a `--version` line answered with the version
//! [`Command`] carries, every answer on stdout.
//!
//! The stdio pipe is the authorisation boundary, so the token is never
//! handed to another process the way a shared agent would. On exit the
//! account drops and its secrets are zeroized.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::with_code, printer::Printer, repl::Repl};

use crate::{
    cli::{Command, Session},
    error::{self, ErrorCode},
};

/// Start a session running commands over one account.
///
/// Each line is a command without the binary name (`token show`, `auth
/// get`), the account token read from storage on first use and held in
/// memory, so the keyring is unlocked once instead of on every command.
/// The account is the session's, and `--json` on a line switches that
/// line alone to JSON. `quit`, `exit`, or EOF ends it.
#[derive(Debug, Parser)]
pub struct ReplCommand;

impl ReplCommand {
    /// Runs the read-eval-print loop until EOF or a quit command, keeping
    /// the session's account across lines, each line printing as
    /// `printer` does unless it asks for JSON.
    pub fn execute(self, printer: &impl Printer, session: &mut Session) -> Result<()> {
        // NOTE: taken up front, so a missing configuration or account
        // fails the session rather than its first line.
        session.account()?;

        let repl = Repl::new("ortie", printer.is_json());

        repl.run::<Command, ErrorCode>(|printer, cmd| {
            cmd.execute(printer, session)
                .map_err(|err| with_code(err, error::code))
        })
    }
}
