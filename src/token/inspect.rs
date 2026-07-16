//! `token inspect` subcommand: print metadata about the access token.

use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use humantime::format_duration;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use io_oauth::rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams;

use crate::account::Account;

/// Inspect metadata associated to the access token.
///
/// Unlike the `token show` command, this command shows you metadata
/// like the token type, when it was issued, when it expires, the
/// presence of a refresh token, and the granted scopes.
#[derive(Debug, Parser)]
pub struct TokenInspectCommand;

impl TokenInspectCommand {
    /// Reads the token from storage and prints its metadata.
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        let response = account.read_from_storage()?;
        printer.out(Report(response))
    }
}

/// Printable metadata view over the stored token response.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Report(Oauth20AccessTokenSuccessParams);

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token type: {}", self.0.token_type.to_lowercase())?;

        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        if let (Some(issued_at), Some(now)) = (self.0.issued_at, now_epoch) {
            let elapsed = Duration::from_secs(now.saturating_sub(issued_at));
            writeln!(f)?;
            write!(f, "Issued: {} ago", format_duration(elapsed))?;
        }

        match self.0.expires_in {
            None => {
                writeln!(f)?;
                write!(f, "Expired: unknown")?;
            }
            Some(exp) => {
                let remaining = match (self.0.issued_at, now_epoch) {
                    (Some(issued_at), Some(now)) => (issued_at + exp as u64).saturating_sub(now),
                    _ => exp as u64,
                };
                writeln!(f)?;
                if remaining == 0 {
                    write!(f, "Expired: true")?;
                } else {
                    let duration = format_duration(Duration::from_secs(remaining));
                    write!(f, "Expires in: {duration}")?;
                }
            }
        }

        writeln!(f)?;
        write!(f, "With refresh token: {}", self.0.refresh_token.is_some())?;

        if let Some(scope) = &self.0.scope {
            writeln!(f)?;
            write!(f, "With scope: {scope}")?;
        }

        Ok(())
    }
}
