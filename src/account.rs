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

use pimalaya_toolbox::{secret::Secret, stream::Tls};
use serde::{Deserialize, Serialize};

use crate::{endpoint::Endpoints, hook::Hooks, storage::Storages};

/// The account configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Account {
    #[serde(default)]
    pub default: bool,

    pub client_id: String,
    pub client_secret: Option<Secret>,

    pub endpoints: Endpoints,
    #[serde(default)]
    pub tls: Tls,

    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub pkce: bool,

    #[serde(default)]
    pub auto_refresh: bool,

    pub storage: Storages,

    #[serde(default)]
    pub on_issue_access_token: Hooks,
    #[serde(default)]
    pub on_refresh_access_token: Hooks,
}
