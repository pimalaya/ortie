//! `token show` subcommand: print the current access token.

use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::{account::Account, token::refresh::TokenRefreshCommand};

/// Seconds of slack before the real expiry at which a token is treated
/// as expired, so a token that is about to lapse is refreshed rather
/// than handed out and rejected mid-request.
const EXPIRY_SKEW_SECS: u64 = 60;

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
    /// Reads the token from storage, refreshing it first when expired
    /// and auto-refresh is requested, then prints it raw.
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        let mut token = account.read_from_storage()?;

        if (self.auto_refresh || account.auto_refresh)
            && let Some(refresh_token) = token.refresh_token.clone()
            && is_expired(token.issued_at, token.expires_in)
        {
            token = TokenRefreshCommand::refresh(account, refresh_token)?;
        }

        printer.out(AccessToken {
            access_token: token.access_token.expose_secret(),
        })
    }
}

/// Whether the token has reached (or is within [`EXPIRY_SKEW_SECS`] of)
/// its real expiry, computed from the issuance time plus its lifetime.
///
/// `expires_in` alone is the lifetime granted at issuance (e.g. 3599s),
/// not a live countdown, so it must be added to `issued_at` and
/// compared against the wall clock. When either is unknown, the token
/// is assumed still valid (refreshing blindly would defeat caching).
fn is_expired(issued_at: Option<u64>, expires_in: Option<usize>) -> bool {
    let (Some(issued_at), Some(expires_in)) = (issued_at, expires_in) else {
        return false;
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    issued_at + expires_in as u64 <= now + EXPIRY_SKEW_SECS
}

/// Printable raw access token, exposed for piping.
#[derive(Debug, Serialize)]
pub struct AccessToken<'a> {
    /// The raw access token string.
    pub access_token: &'a str,
}

impl fmt::Display for AccessToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.access_token)
    }
}
