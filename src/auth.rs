//! `auth` subcommand tree: obtain OAuth 2.0 access tokens by running
//! the grant configured on the account.

pub mod get;
pub mod resume;

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    auth::{get::AuthGetCommand, resume::AuthResumeCommand},
    cli::take_account,
};

/// Get a fresh access token by running the account's OAuth grant.
///
/// Start an authorization-code, device or client-credentials grant,
/// or resume one with a redirected URI or device code.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Get(AuthGetCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl AuthCommand {
    /// Dispatches the auth leaf, resolving the account it runs against.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Get(cmd) => {
                let mut account = take_account(printer, config_paths, account_name)?;
                cmd.execute(printer, &mut account)
            }
            Self::Resume(cmd) => {
                let mut account = take_account(printer, config_paths, account_name)?;
                cmd.execute(printer, &mut account)
            }
        }
    }
}
