//! # Command-line interface
//!
//! The root clap parser of the `ortie` binary, its subcommand router,
//! and the account resolution every account-bound command starts from.

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, ConfigFlags, JsonFlag, LogFlags},
        commands::{
            completion::CompletionCommand, json_schema::JsonSchemaCommand, manual::ManualCommand,
        },
    },
    error::NoErrorCode,
    footer, long_version,
    mcp::exit_on_failure,
    printer::Printer,
    shell::invocation,
    welcome::{Welcome, is_wanted},
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};

use crate::{
    account::Account, auth::AuthCommand, config::Config, json_schema, mcp::McpCommand,
    repl::ReplCommand, token::TokenCommand,
};

/// The sample configuration as a raw file, for `curl` to fetch.
const SAMPLE_RAW_URL: &str =
    "https://raw.githubusercontent.com/pimalaya/ortie/master/config.sample.toml";

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
    pub config: ConfigFlags,
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
                let invocation = invocation("ortie", config_paths, account_name);
                cmd.execute(&*printer, account, &invocation)
            }
            Self::Mcp(cmd) => {
                // stdout carries protocol messages only, so a failure goes
                // to stderr, a person missing a configuration aside.
                let account = match take_account(config_paths, account_name) {
                    Err(err) if err.is::<ConfigMissingError>() && is_wanted(&*printer) => {
                        return Err(err);
                    }
                    account => exit_on_failure::<NoErrorCode, _>(account),
                };

                exit_on_failure::<NoErrorCode, _>(cmd.execute(account));
                Ok(())
            }
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}

/// Meets a bare `ortie`: the help when a configuration exists, the
/// missing configuration otherwise, which a person meets as the welcome.
pub fn meet_bare_invocation(config_paths: &[PathBuf]) -> Result<()> {
    // NOTE: a broken file counts as a configuration, its error left to
    // the first command reading it.
    if let Err(err) = Config::load(config_paths)
        && err.is::<ConfigMissingError>()
    {
        return Err(err);
    }

    Cli::command().print_help()?;
    Ok(())
}

/// Takes the named (or default) account out of the config at
/// `config_paths`, flattened into its runtime view.
pub(crate) fn take_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<Account> {
    let (_, account) = Config::load(config_paths)?.take_account(account_name)?;
    Ok(Account::from(account))
}

/// The welcome a person meets when no configuration exists at `path`:
/// the commands creating it from `sample`, the shipped sample
/// configuration, or downloading it without one, then `auth get`.
pub fn welcome<'a>(
    path: &'a Path,
    config_paths: &'a [PathBuf],
    sample: Option<&'a Path>,
) -> Welcome<'a> {
    Welcome {
        bin: "ortie",
        intro: "Welcome to Ortie, the CLI to manage OAuth tokens.",
        path,
        config_paths,
        sample,
        sample_url: SAMPLE_RAW_URL,
        next: "Edit it and uncomment the options you need, then get a first token:",
        command: "auth get",
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use pimalaya_config::toml::{AccountError, ConfigMissingError};

    use crate::cli::take_account;

    #[test]
    fn a_missing_configuration_is_typed_for_the_welcome() {
        let paths = [PathBuf::from("/nonexistent/ortie/config.toml")];
        let err = take_account(&paths, None).unwrap_err();
        let missing = err.downcast_ref::<ConfigMissingError>().unwrap();
        assert_eq!(missing.path, paths[0]);
    }

    /// Ortie reports which accounts are marked default, so several are
    /// refused rather than one taken at random.
    #[test]
    fn several_default_accounts_are_refused_naming_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let account = |name: &str| {
            format!(
                "[accounts.{name}]\ndefault = true\nclient-id = \"c\"\n\
                 storage.read.command = \"true\"\nstorage.write.command = \"true\"\n"
            )
        };
        fs::write(&path, account("b") + &account("a")).unwrap();

        let err = take_account(&[path], None).unwrap_err();

        match err.downcast_ref::<AccountError>() {
            Some(AccountError::SeveralDefaults(names)) => assert_eq!(names, &["a", "b"]),
            _ => panic!("{err:?}"),
        }
    }
}
