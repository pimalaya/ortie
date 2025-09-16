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

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use super::{get::GetAuthorizationCommand, resume::ResumeAuthorizationCommand};

/// Get a fresh new OAuth 2.0 access token by initiating or resuming
/// an Authorization Code Grant flow.
///
/// This subcommand allows you to get an authorization URI that needs
/// to be opened by the user, or to resume an authorization flow with
/// an existing redirect URI.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Get(GetAuthorizationCommand),
    Resume(ResumeAuthorizationCommand),
}

impl AuthCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}
