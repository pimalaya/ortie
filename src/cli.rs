use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use pimalaya_tui::{
    long_version,
    terminal::{
        cli::{arg::path_parser, printer::Printer},
        config::TomlConfig as _,
    },
};

use crate::{
    auth::Auth, completion::GenerateCompletionScripts, config::Config, manual::GenerateManuals,
    token::Token,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: OrtieCommand,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). If the first path does not point to a valid file,
    /// the wizard will propose to assist you in the creation of the
    /// configuration file. Other paths are merged with the first one,
    /// which allows you to separate your public config from your
    /// private(s) one(s).
    #[arg(short, long = "config", global = true, env = "ORTIE_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser)]
    pub config_paths: Vec<PathBuf>,

    /// Override the default account.
    ///
    /// An account name corresponds to an entry in the table at the
    /// root level of your TOML configuration file.
    #[arg(short, long = "account", global = true, env = "ORTIE_ACCOUNT")]
    #[arg(value_name = "NAME")]
    pub account_name: Option<String>,

    /// Enable JSON output.
    ///
    /// When set, command output (data and errors) is displayed as
    /// JSON string.
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable all logs.
    ///
    /// Same as running command with `RUST_LOG=off` environment
    /// variable.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "debug")]
    #[arg(conflicts_with = "trace")]
    pub quiet: bool,

    /// Enable debug logs.
    ///
    /// Same as running command with `RUST_LOG=debug` environment
    /// variable.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "quiet")]
    #[arg(conflicts_with = "trace")]
    pub debug: bool,

    /// Enable verbose trace logs with backtrace.
    ///
    /// Same as running command with `RUST_LOG=trace` and
    /// `RUST_BACKTRACE=1` environment variables.
    #[arg(long, global = true)]
    #[arg(conflicts_with = "quiet")]
    #[arg(conflicts_with = "debug")]
    pub trace: bool,
}

#[derive(Subcommand, Debug)]
pub enum OrtieCommand {
    #[command(subcommand)]
    #[command(arg_required_else_help = true)]
    Auth(Auth),

    #[command(subcommand)]
    #[command(arg_required_else_help = true)]
    Token(Token),

    #[command(arg_required_else_help = true)]
    #[command(alias = "mans")]
    Manuals(GenerateManuals),
    #[command(arg_required_else_help = true)]
    Completions(GenerateCompletionScripts),
}

impl OrtieCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Auth(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (_, account) = config.to_toml_account_config(account_name)?;
                cmd.execute(printer, account)
            }
            Self::Token(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (_, account) = config.to_toml_account_config(account_name)?;
                cmd.execute(printer, account)
            }
            Self::Manuals(cmd) => cmd.execute(printer),
            Self::Completions(cmd) => cmd.execute(printer),
        }
    }
}
