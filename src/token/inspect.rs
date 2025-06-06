use std::{fmt, time::Duration};

use anyhow::Result;
use clap::Parser;
use humantime::format_duration;
use io_oauth::v2_0::IssueAccessTokenSuccessParams;
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::account::Account;

/// Inspect access token.
#[derive(Debug, Parser)]
pub struct InspectToken;

impl InspectToken {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let response = account.storage.read()?;
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
