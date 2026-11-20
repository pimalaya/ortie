//! # Command-line interface
//!
//! The root clap parser of the `ortie` binary, its subcommand router,
//! and the session every account-bound command takes its account from.

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
    error::with_code,
    footer, long_version,
    mcp::exit_on_failure,
    printer::Printer,
    shell::invocation,
    welcome::{Welcome, is_wanted},
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};

use crate::{
    account::{Account, AccountCommand},
    auth::AuthCommand,
    config::Config,
    error::{self, ErrorCode},
    json_schema,
    mcp::McpCommand,
    repl::ReplCommand,
    token::TokenCommand,
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

/// The top-level commands: the token and grant trees, the sessions, the
/// accounts, then the standard commands.
///
/// The version it carries is what a `-V` or `--version` line of a session
/// prints, as the command line does.
#[derive(Subcommand, Debug)]
#[command(version, long_version = long_version!())]
pub enum Command {
    #[command(subcommand)]
    Auth(AuthCommand),
    #[command(subcommand)]
    Token(TokenCommand),
    Repl(ReplCommand),
    Mcp(McpCommand),
    #[command(subcommand, alias = "accounts")]
    Account(AccountCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(aliases = ["manuals", "mans"])]
    Manual(ManualCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

impl Command {
    /// Dispatches the parsed subcommand over `session`, which takes the
    /// account on first use.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        match self {
            Self::Auth(cmd) => {
                let invocation = session.invocation();
                let interrupt_handoff = session.interrupt_handoff;
                cmd.execute(printer, session.account()?, &invocation, interrupt_handoff)
            }
            Self::Token(cmd) => cmd.execute(printer, session.account()?),
            Self::Repl(cmd) => cmd.execute(&*printer, session),
            Self::Mcp(cmd) => {
                // stdout carries protocol messages only, so a failure goes
                // to stderr, a person missing a configuration aside.
                let account = match session.account.take() {
                    Some(account) => Ok(account),
                    None => take_account(session.config_paths, session.account_name),
                };

                let account = match account {
                    Err(err) if err.is::<ConfigMissingError>() && is_wanted(&*printer) => {
                        return Err(err);
                    }
                    account => exit_on_failure::<ErrorCode, _>(account),
                };

                exit_on_failure::<ErrorCode, _>(
                    cmd.execute(account)
                        .map_err(|err| with_code(err, error::code)),
                );
                Ok(())
            }
            Self::Account(cmd) => cmd.execute(printer, session),
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}

/// What the commands of one run share: the configuration paths, the
/// account `-a` names, and that account once taken, which a REPL session
/// keeps across its lines.
pub struct Session<'a> {
    /// The configuration paths, `-c` or the default ones.
    pub config_paths: &'a [PathBuf],
    /// The account `-a` names, the default one when `None`.
    pub account_name: Option<&'a str>,
    /// The account, taken on first use.
    pub account: Option<Account>,
    /// Whether an interactive `auth get` interrupted while it waits prints
    /// its handoff, which a REPL session leaves to the default signal end.
    pub interrupt_handoff: bool,
}

impl Session<'_> {
    /// The session's account, taken from the configuration on first use.
    pub fn account(&mut self) -> Result<&mut Account> {
        let account = match self.account.take() {
            Some(account) => account,
            None => take_account(self.config_paths, self.account_name)?,
        };

        Ok(self.account.insert(account))
    }

    /// The binary with the `-c` and `-a` of the session, which `auth get`
    /// starts the commands of its handoff with.
    pub fn invocation(&self) -> String {
        invocation("ortie", self.config_paths, self.account_name)
    }
}

/// Takes the named (or default) account out of the config at
/// `config_paths`, flattened into its runtime view.
pub(crate) fn take_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<Account> {
    let (name, account) = Config::load(config_paths)?.take_account(account_name)?;
    Ok(Account::new(name, account))
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
