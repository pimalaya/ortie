use std::{fmt, time::Duration};

use anyhow::Result;
use clap::Parser;
use humantime::format_duration;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::{cli::account::Account, issue_access_token::IssueAccessTokenSuccessParams};

/// Inspect metadata associated to the access token.
///
/// Unlike the `token show` command, this command shows you metadata
/// like the token type, when it was issued, when it expires, the
/// presence of a refresh token, and the granted scopes.
#[derive(Debug, Parser)]
pub struct TokenInspectCommand;

impl TokenInspectCommand {
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        let response = account.read_from_storage()?;
        printer.out(Report(response))
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Report(IssueAccessTokenSuccessParams);

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token type: {}", self.0.token_type.to_lowercase())?;

        if let Ok(mut elapsed) = self.0.issued_at.elapsed() {
            elapsed = Duration::from_secs(elapsed.as_secs());
            writeln!(f)?;
            write!(f, "Issued: {} ago", format_duration(elapsed))?;
        }

        match self.0.expires_in {
            None => {
                writeln!(f)?;
                write!(f, "Expired: unknown")?;
            }
            Some(0) => {
                writeln!(f)?;
                write!(f, "Expired: true")?;
            }
            Some(exp) => {
                let duration = Duration::from_secs(exp as u64);
                let duration = format_duration(duration);
                writeln!(f)?;
                write!(f, "Expires in: {duration}")?;
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
