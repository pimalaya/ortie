use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use pimalaya_toolbox::{
    long_version,
    terminal::{
        cli::{AccountFlag, ConfigPathsFlag, JsonFlag, LogFlags},
        config::TomlConfig,
        printer::Printer,
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
    pub command: Ortie,
    #[command(flatten)]
    pub config: ConfigPathsFlag,
    #[command(flatten)]
    pub account: AccountFlag,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Subcommand, Debug)]
pub enum Ortie {
    #[command(arg_required_else_help = true, subcommand)]
    Auth(Auth),
    #[command(arg_required_else_help = true, subcommand)]
    Token(Token),
    #[command(arg_required_else_help = true, alias = "mans")]
    Manuals(GenerateManuals),
    #[command(arg_required_else_help = true)]
    Completions(GenerateCompletionScripts),
}

impl Ortie {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Auth(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (_, account) = config.get_account(account_name)?;
                cmd.execute(printer, account)
            }
            Self::Token(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (_, account) = config.get_account(account_name)?;
                cmd.execute(printer, account)
            }
            Self::Manuals(cmd) => cmd.execute(printer),
            Self::Completions(cmd) => cmd.execute(printer),
        }
    }
}
