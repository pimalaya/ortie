//! # Account configuration
//!
//! Module dedicated to account configuration.

use serde::{Deserialize, Serialize};

use crate::{endpoint::Endpoints, hook::Hooks, secret::Secret, storage::Storages, stream::Tls};

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
