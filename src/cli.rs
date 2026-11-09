//! # Command-line interface
//!
//! The root clap parser of the `ortie` binary, its subcommand router,
//! and the account resolution every account-bound command starts from.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, JsonSchemaCommand, ManualCommand},
        parsers::path_parser,
    },
    footer, long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

use crate::{
    account::Account,
    auth::{AuthCommand, get::invocation},
    config::Config,
    json_schema,
    mcp::McpCommand,
    repl::ReplCommand,
    token::TokenCommand,
};

/// Top-level command-line interface for the `ortie` binary.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about, long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run, bare `ortie` showing the help.
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
    Repl(ReplCommand),
    Mcp(McpCommand),
    #[command(aliases = ["manuals", "mans"])]
    Manual(ManualCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

impl Command {
    /// Dispatches the parsed subcommand, resolving the account for the
    /// token tree (the auth tree resolves it per leaf).
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Auth(cmd) => cmd.execute(printer, config_paths, account_name),
            Self::Token(cmd) => {
                let mut account = take_account(config_paths, account_name)?;
                cmd.execute(printer, &mut account)
            }
            Self::Repl(cmd) => {
                let account = take_account(config_paths, account_name)?;
                let invocation = invocation(config_paths, account_name);
                cmd.execute(printer, account, &invocation)
            }
            Self::Mcp(cmd) => {
                let account = take_account(config_paths, account_name)?;
                cmd.execute(account)
            }
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}

/// Takes the named (or default) account out of the config at
/// `config_paths`, flattened into its runtime view.
pub(crate) fn take_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<Account> {
    let Some(mut config) = Config::from_paths_or_default(config_paths)? else {
        // NOTE: where `-c` pointed, or the default location when it
        // named none, so a mistyped path shows up as itself.
        let path = Config::target_path(config_paths)?;

        bail!(
            "No configuration found at {}, start from a sample: \
             https://github.com/pimalaya/ortie/tree/master/samples",
            path.display(),
        );
    };

    // NOTE: an empty name and `default` both mean the default account,
    // which the block below resolves.
    let named = account_name.filter(|name| !name.is_empty() && *name != "default");

    if let Some(name) = named.filter(|name| !config.accounts.contains_key(*name)) {
        let mut names: Vec<&str> = config.accounts.keys().map(String::as_str).collect();
        names.sort_unstable();

        bail!(
            "Account `{name}` not found, the configuration holds: {}",
            names.join(", "),
        );
    }

    let Some((_, account)) = config.take_account(account_name)? else {
        bail!(
            "No default account found, name one with `-a <NAME>` or mark one with `default = true`"
        );
    };

    Ok(Account::from(account))
}

/// Path(s) to the TOML configuration file(s).
#[derive(Debug, Default, Parser)]
pub struct ConfigPathsArg {
    /// Override the default configuration file path.
    ///
    /// Paths are shell-expanded, then canonicalized when applicable.
    /// Several can be given at once, delimited by `:` like `$PATH` in a
    /// POSIX shell: the first one is the base and the rest are merged on
    /// top, which is how a public config and a private one combine.
    #[arg(long = "config", short = 'c', global = true, env = "ORTIE_CONFIG")]
    #[arg(name = "config_paths", value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub paths: Vec<PathBuf>,
}
