use std::collections::HashMap;

use pimalaya_toolbox::config::TomlConfig;
use serde::{Deserialize, Serialize};

use crate::account::Account;

/// The main configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, Account>,
}

impl TomlConfig for Config {
    type Account = Account;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn find_default_account(&self) -> Option<(String, Self::Account)> {
        self.accounts
            .iter()
            .find(|(_, account)| account.default)
            .map(|(name, account)| (name.to_owned(), account.clone()))
    }

    fn find_account(&self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }
}
