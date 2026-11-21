//! # Account list command
//!
//! The `account list` subcommand, listing the accounts a configuration
//! declares, each with its grant.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use pimalaya_config::toml::TomlConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::config::{Config, GrantConfig};

/// List all accounts declared in the configuration.
///
/// One line per account, sorted by name: its name, its grant, and
/// `default` on the one a command passing no `-a <NAME>` takes. Nothing
/// is reached: this reads the configuration only, so it runs whatever
/// `-a` names and however many accounts are marked default. Exits 0, or
/// 1 on a configuration it cannot read.
///
/// JSON output: `{"accounts": [{"name", "default", "grant"}]}`, `grant`
/// as the configuration spells it (`authorization-code`, `device`,
/// `client-credentials` or `client-credentials-jwt`).
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    /// Loads the configuration, taking no account, and prints its
    /// accounts sorted by name.
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = Config::load(config_paths)?;

        let mut accounts: Vec<AccountListRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountListRow {
                name: name.clone(),
                default: account.default,
                grant: account.grant,
            })
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}

/// The `account list` output, the accounts the configuration declares.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountListOutput {
    /// The accounts, sorted by name.
    pub accounts: Vec<AccountListRow>,
}

/// One line per account: its name, its grant and the default marker.
impl fmt::Display for AccountListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::default();

        for account in &self.accounts {
            columns.push([
                sanitize::text(&account.name).into_owned(),
                String::from(account.grant.as_str()),
                String::from(if account.default { "default" } else { "" }),
            ]);
        }

        write!(f, "{columns}")
    }
}

/// One account of the list.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountListRow {
    /// The name of the `[accounts.<name>]` block.
    pub name: String,
    /// Whether a command with no `-a <NAME>` runs against it.
    pub default: bool,
    /// The OAuth 2.0 grant its auth commands run.
    pub grant: GrantConfig,
}
