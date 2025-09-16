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
    const DEFAULT_REDIRECTION_SCHEME: &'static str = "http";
    const DEFAULT_REDIRECTION_HOST: &'static str = "127.0.0.1";

    pub fn redirection_scheme(&self) -> &str {
        let Some(uri) = &self.redirection else {
            return Self::DEFAULT_REDIRECTION_SCHEME;
        };

        uri.scheme()
    }

    pub fn redirection_host(&self) -> &str {
        let Some(uri) = &self.redirection else {
            return Self::DEFAULT_REDIRECTION_HOST;
        };

        let Some(host) = uri.host_str() else {
            return Self::DEFAULT_REDIRECTION_HOST;
        };

        host
    }

    pub fn redirection_port(&self) -> u16 {
        let Some(uri) = &self.redirection else {
            return 80;
        };

        let Some(port) = uri.port_or_known_default() else {
            return if uri.scheme().eq_ignore_ascii_case("https") {
                443
            } else {
                80
            };
        };

        port
    }
}
