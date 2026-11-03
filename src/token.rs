//! # Token commands
//!
//! The `token` subcommand tree, working on the access token already
//! persisted in storage rather than running a grant.

pub mod inspect;
pub mod issued;
pub mod refresh;
pub mod revoke;
pub mod show;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    account::Account,
    token::{
        inspect::TokenInspectCommand, refresh::TokenRefreshCommand, revoke::TokenRevokeCommand,
        show::TokenShowCommand,
    },
};

/// Display, refresh and revoke an existing OAuth 2.0 access token.
///
/// Show the access token, inspect the metadata associated to it,
/// refresh it with the refresh token when one is available, or revoke
/// it at the authorization server.
#[derive(Subcommand, Debug)]
pub enum TokenCommand {
    #[command(visible_alias = "get")]
    Show(TokenShowCommand),
    Inspect(TokenInspectCommand),
    Refresh(TokenRefreshCommand),
    Revoke(TokenRevokeCommand),
}

impl TokenCommand {
    /// Dispatches the token leaf on the resolved account.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        match self {
            Self::Show(cmd) => cmd.execute(printer, account),
            Self::Inspect(cmd) => cmd.execute(printer, account),
            Self::Refresh(cmd) => cmd.execute(printer, account),
            Self::Revoke(cmd) => cmd.execute(printer, account),
        }
    }
}
