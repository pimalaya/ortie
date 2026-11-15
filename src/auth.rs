//! # Auth commands
//!
//! The `auth` subcommand tree, obtaining OAuth 2.0 access tokens by
//! running the grant configured on the account.

pub mod get;
pub mod listen;
pub mod resume;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    account::Account,
    auth::{get::AuthGetCommand, listen::AuthListenCommand, resume::AuthResumeCommand},
};

/// Get a fresh access token by running the account's OAuth grant.
///
/// Start an authorization-code, device or client-credentials grant, listen
/// for the redirection of a handed-off one, or resume one with a redirected
/// URI or a device code.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Get(AuthGetCommand),
    Listen(AuthListenCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl AuthCommand {
    /// Dispatches the auth leaf on the resolved account, `invocation`
    /// starting the commands an `auth get` handoff prints.
    pub fn execute(
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
