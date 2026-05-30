//! `token` subcommand router (show, refresh, inspect).

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::cli::{
    account::Account, token_inspect::TokenInspectCommand, token_refresh::TokenRefreshCommand,
    token_show::TokenShowCommand,
};

/// Display and refresh an existing OAuth 2.0 access token.
///
/// This subcommand allows you to show your access token, inspect
/// metadata associated to it, and refresh your access token using the
/// refresh token (if available).
#[derive(Subcommand, Debug)]
pub enum TokenCommand {
    #[command(visible_alias = "get")]
    Show(TokenShowCommand),
    Inspect(TokenInspectCommand),
    Refresh(TokenRefreshCommand),
}

impl TokenCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Show(cmd) => cmd.execute(printer, account),
            Self::Inspect(cmd) => cmd.execute(printer, account),
            Self::Refresh(cmd) => cmd.execute(printer, account),
        }
    }
}
