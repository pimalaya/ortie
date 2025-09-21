// This file is part of Ortie, a CLI to manage OAuth 2.0 access
// tokens.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::{fmt, time::Duration};

use anyhow::Result;
use clap::Parser;
use humantime::format_duration;
use io_oauth::v2_0::issue_access_token::IssueAccessTokenSuccessParams;
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::account::Account;

/// Inspect metadata associated to the access token.
///
/// Unlike the `token get` command, this command shows you metadata
/// like the access token, the refresh token, when it was issued and
/// when it expires.
#[derive(Debug, Parser)]
pub struct InspectTokenCommand;

impl InspectTokenCommand {
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
