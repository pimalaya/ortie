//! # Token show command
//!
//! The `token show` subcommand, printing the stored access token raw so
//! it pipes into whatever needed it, and refreshing it first when the
//! account or the flag asks for a fresh one.

use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::{
    account::Account,
    token::refresh::{RefreshAction, TokenRefreshCommand, refresh_action},
};

/// Slack before the real expiry at which a token counts as expired, so
/// one about to lapse is refreshed rather than rejected mid-request.
const EXPIRY_SKEW_SECS: u64 = 60;

/// Access token lifetime assumed when the server sends no `expires_in`,
/// so a session refreshes roughly hourly instead of never.
const DEFAULT_EXPIRES_IN_SECS: u64 = 3600;

/// Display the raw access token.
///
/// The token is printed alone, so it pipes straight into whatever
/// application needs it.
#[derive(Debug, Parser)]
pub struct TokenShowCommand {
    /// Automatically refresh the access token when expired.
    ///
    /// Guarantees a fresh access token. The `auto-refresh` config
    /// option turns it on for every call.
    #[arg(long, short = 'r')]
    pub auto_refresh: bool,
}

impl TokenShowCommand {
    /// Reads the token from storage, making it fresh first when
    /// auto-refresh is on, then prints it raw.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        let auto_refresh = self.auto_refresh || account.auto_refresh;

        // NOTE: a missing or unreadable token re-acquires rather than
        // fails on an auto-refreshing client credentials account, so
        // the very first run needs no prior auth get.
        let mut token = match account.resolve_token() {
            Ok(token) => token,
            Err(_)
                if auto_refresh
                    && refresh_action(account.grant, false) == RefreshAction::Reacquire =>
            {
                TokenRefreshCommand::reacquire(account)?
            }
            Err(err) => return Err(err),
        };

        if auto_refresh && is_expired(token.issued_at, token.expires_in) {
            match refresh_action(account.grant, token.refresh_token.is_some()) {
                RefreshAction::Reacquire => {
                    token = TokenRefreshCommand::reacquire(account)?;
                }
                RefreshAction::Refresh => {
                    if let Some(refresh_token) = token.refresh_token.clone() {
                        token = TokenRefreshCommand::refresh(account, refresh_token)?;
                    }
                }
                RefreshAction::Keep => (),
            }
        }

        printer.out(TokenShowOutput {
            access_token: token.access_token.expose_secret(),
        })
    }
}

/// Whether the token reached its real expiry, or is within
/// [`EXPIRY_SKEW_SECS`] of it.
///
/// `expires_in` is the lifetime granted at issuance, not a countdown,
/// so it is added to `issued_at`. An unknown `issued_at` counts as
/// valid, a missing `expires_in` as [`DEFAULT_EXPIRES_IN_SECS`].
fn is_expired(issued_at: Option<u64>, expires_in: Option<usize>) -> bool {
    let Some(issued_at) = issued_at else {
        return false;
    };
    let expires_in = expires_in
        .map(|exp| exp as u64)
        .unwrap_or(DEFAULT_EXPIRES_IN_SECS);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    issued_at + expires_in <= now + EXPIRY_SKEW_SECS
}

/// Printable raw access token, exposed for piping.
#[derive(Debug, JsonSchema, Serialize)]
pub struct TokenShowOutput<'a> {
    /// The raw access token string.
    pub access_token: &'a str,
}

impl fmt::Display for TokenShowOutput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.access_token)
    }
}
