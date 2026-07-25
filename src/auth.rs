//! `auth` subcommand tree: obtain OAuth 2.0 access tokens by running
//! the grant configured on the account.

pub mod discover;
pub mod get;
pub mod resume;

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    auth::{discover::AuthDiscoverCommand, get::AuthGetCommand, resume::AuthResumeCommand},
    cli::take_account,
};

/// Discover OAuth 2.0 services or run the account's OAuth grant.
///
/// Discover an issuer, start an authorization-code or device grant, or
/// resume with a redirected URI or device code.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Discover(AuthDiscoverCommand),
    Get(AuthGetCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl AuthCommand {
    /// Dispatches the auth leaf, resolving the account for the leaves
    /// that need one (discover runs before any account exists).
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Discover(cmd) => cmd.execute(printer),
            Self::Get(cmd) => cmd.execute(printer, take_account(config_paths, account_name)?),
            Self::Resume(cmd) => cmd.execute(printer, take_account(config_paths, account_name)?),
        }
    }
}
