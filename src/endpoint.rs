// This file is part of Ortie, a CLI to manage OAuth tokens.
//
// Copyright (C) 2025-2026 Clément DOUIN <pimalaya.org@posteo.net>
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

use std::{borrow::Cow, net::TcpListener};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoints {
    pub authorization: Url,
    pub token: Url,
    pub redirection: Option<Url>,
}

impl Endpoints {
    pub fn redirection(&self) -> Result<Cow<'_, Url>> {
        if let Some(url) = self.redirection.as_ref() {
            return Ok(Cow::Borrowed(url));
        }

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let url: Url = format!("http://127.0.0.1:{port}").parse()?;

        Ok(Cow::Owned(url))
    }
}
