//! # Account list command
//!
//! The `account list` subcommand, listing the accounts a configuration
//! declares as pimalaya-cli's shared [`AccountListOutput`] prints them.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    account::{AccountListOutput, AccountRow},
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

use crate::config::Config;

/// List all accounts declared in the configuration.
///
/// One line per account: its name, and `default` on the one a command
/// passing no `-a <NAME>` takes. Nothing is reached: this reads the
/// configuration only.
///
/// JSON output: `{"accounts": [{"name", "default", "backends",
/// "notBuilt"}]}`, the two lists always empty, Ortie having no backend.
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    /// Loads the configuration, taking no account, and prints its
    /// accounts sorted by name.
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = Config::load(config_paths)?;

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountRow {
                name: name.clone(),
                default: account.default,
                backends: Vec::new(),
                not_built: Vec::new(),
            })
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}
