use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::cli::{account::Account, auth_get::AuthGetCommand, auth_resume::AuthResumeCommand};

/// Get a fresh new OAuth 2.0 access token by initiating or resuming
/// an Authorization Code Grant flow.
///
/// This subcommand allows you to get an authorization URI that needs
/// to be opened by the user, or to resume an authorization flow with
/// an existing redirect URI.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Get(AuthGetCommand),
    Resume(AuthResumeCommand),
}

impl AuthCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}
