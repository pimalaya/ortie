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

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::account::Account;

use super::refresh::RefreshTokenCommand;

/// Display the raw access token.
///
/// This command allows you to see your access token. It can easily be
/// piped to other applications.
#[derive(Debug, Parser)]
pub struct ShowTokenCommand {
    /// Automatically refresh the access token when expired.
    ///
    /// This option insures you that you get a fresh access token. See
    /// also the `auto-refresh` config option.
    #[arg(long, short = 'r')]
    pub auto_refresh: bool,
}

impl ShowTokenCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut token = account.storage.read()?;

        if self.auto_refresh || account.auto_refresh {
            if let Some(refresh_token) = token.refresh_token {
                if let Some(0) = token.expires_in {
                    token = RefreshTokenCommand::refresh(account, refresh_token)?;
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
