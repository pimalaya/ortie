use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::cli::{account::Account, token_refresh::TokenRefreshCommand};

/// Display the raw access token.
///
/// This command allows you to see your access token. It can easily be
/// piped to other applications.
#[derive(Debug, Parser)]
pub struct TokenShowCommand {
    /// Automatically refresh the access token when expired.
    ///
    /// This option insures you that you get a fresh access token. See
    /// also the `auto-refresh` config option.
    #[arg(long, short = 'r')]
    pub auto_refresh: bool,
}

impl TokenShowCommand {
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        let mut token = account.read_from_storage()?;

        if self.auto_refresh || account.auto_refresh {
            if let Some(refresh_token) = token.refresh_token {
                if let Some(0) = token.expires_in {
                    token = TokenRefreshCommand::refresh(account, refresh_token)?;
                }
            }
        }

        printer.out(AccessToken {
            access_token: token.access_token.expose_secret(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AccessToken<'a> {
    pub access_token: &'a str,
}

impl fmt::Display for AccessToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.access_token)
    }
}
