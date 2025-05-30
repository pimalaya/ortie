use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::account::Account;

/// The main configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, Account>,
}

impl pimalaya_tui::terminal::config::TomlConfig for Config {
    type TomlAccountConfig = Account;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn get_default_account_config(&self) -> Option<(String, Self::TomlAccountConfig)> {
        self.accounts
            .iter()
            .find(|(_, account)| account.default)
            .map(|(name, account)| (name.to_owned(), account.clone()))
    }

    fn get_account_config(&self, name: &str) -> Option<(String, Self::TomlAccountConfig)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }
}
