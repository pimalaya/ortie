//! Root clap parser for the `ortie` binary.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

use crate::{account::Account, auth::AuthCommand, config::Config, token::TokenCommand};

/// Top-level command-line interface for the `ortie` binary.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version)]
#[command(about = "CLI to manage OAuth tokens")]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run; bare `ortie` runs the discovery wizard
    /// (auth discover).
    #[command(subcommand)]
    pub cmd: Option<Command>,

    /// Path(s) to the TOML configuration file(s).
    #[command(flatten)]
    pub config: ConfigPathsArg,

    /// Name of the account to run the subcommand with.
    #[command(flatten)]
    pub account: AccountFlag,

    /// Switch the output format to JSON.
    #[command(flatten)]
    pub json: JsonFlag,

    /// Log level and log file destination.
    #[command(flatten)]
    pub log: LogFlags,
}

/// Top-level subcommand router.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Auth(AuthCommand),
    #[command(subcommand)]
    Token(TokenCommand),

    #[command(alias = "mans")]
    Manuals(ManualCommand),
    Completions(CompletionCommand),
}

impl Command {
    /// Dispatches the parsed subcommand, resolving the account first
    /// for the token tree (the auth tree resolves it per leaf).
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Auth(cmd) => cmd.execute(printer, config_paths, account_name),
            Self::Token(cmd) => {
                let account = take_account(config_paths, account_name)?;
                cmd.execute(printer, account)
            }
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}

/// Loads the config from `config_paths` and takes the named (or
/// default) account out of it, flattened into its runtime view.
pub(crate) fn take_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<Account> {
    let Some(mut config) = Config::from_paths_or_default(config_paths)? else {
        bail!("Config file not found");
    };

    let Some((_, account)) = config.take_account(account_name)? else {
        bail!("Account not found");
    };

    Ok(Account::from(account))
}

/// Path(s) to the TOML configuration file(s).
#[derive(Debug, Default, Parser)]
pub struct ConfigPathsArg {
    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). Other paths are merged with the first one, which
    /// allows you to separate your public config from your private
    /// one(s).
    #[arg(long = "config", short = 'c', global = true)]
    #[arg(name = "config_paths", value_name = "PATH", value_parser = path_parser)]
    pub paths: Vec<PathBuf>,
}
