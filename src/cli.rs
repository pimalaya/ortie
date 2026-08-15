//! Root clap parser for the `ortie` binary.

use std::{
    io::{IsTerminal, stdin},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    footer, long_version,
    printer::Printer,
    prompt,
};
use pimalaya_config::toml::TomlConfig;

use crate::{
    account::Account,
    auth::AuthCommand,
    config::Config,
    repl::ReplCommand,
    token::TokenCommand,
    wizard::{self, CONFIG_SAMPLE_URL, ConfigureCommand},
};

/// Top-level command-line interface for the `ortie` binary.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about, long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The subcommand to run; bare `ortie` runs the configuration
    /// wizard.
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
    /// Configure an account interactively.
    #[command(visible_alias = "wizard")]
    Configure(ConfigureCommand),

    #[command(subcommand)]
    Auth(AuthCommand),
    #[command(subcommand)]
    Token(TokenCommand),

    Repl(ReplCommand),

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
            Self::Configure(cmd) => cmd.execute(printer, config_paths),
            Self::Auth(cmd) => cmd.execute(printer, config_paths, account_name),
            Self::Token(cmd) => {
                let mut account = take_account(printer, config_paths, account_name)?;
                cmd.execute(printer, &mut account)
            }
            Self::Repl(cmd) => {
                let account = take_account(printer, config_paths, account_name)?;
                cmd.execute(printer, account)
            }
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}

/// Welcomes, then offers to generate a first configuration. Returns
/// whether the wizard ran.
///
/// Raised from the two places nothing can happen without a
/// configuration: a bare invocation, and a command that needs an
/// account. It is a hook rather than a gate, so declining it decides
/// nothing: what happens next is the caller's business, and for a
/// command that is simply carrying on.
pub fn offer_configuration(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    path: &Path,
) -> Result<bool> {
    wizard::print_welcome(path);

    if !prompt::bool("Create a configuration with a default account?", true)? {
        return Ok(false);
    }

    ConfigureCommand.execute(printer, config_paths)?;

    Ok(true)
}

/// Loads the config from `config_paths` and takes the named (or
/// default) account out of it, flattened into its runtime view.
///
/// A missing configuration is met with the wizard rather than with an
/// error: the welcome frames what Ortie is and offers to generate an
/// account, then the command carries on either way. Accepting is what
/// gives it a chance to work; declining leaves it to fail on the
/// configuration it still has not got. The two other failures name what
/// is missing and how to pick an account.
pub(crate) fn take_account(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<Account> {
    let mut config = match Config::from_paths_or_default(config_paths)? {
        Some(config) => config,
        None => {
            // NOTE: the target path is where `-c` pointed, or the default
            // location when it named none, so a mistyped path shows up as
            // itself rather than as a generic first run.
            let path = Config::target_path(config_paths)?;

            // NOTE: nobody is there to answer a prompt in a script or a
            // cron job, and a JSON consumer wants a failure it can read,
            // so both skip the offer and fail below.
            if !printer.is_json() && stdin().is_terminal() {
                offer_configuration(printer, config_paths, &path)?;
            }

            // NOTE: the wizard also prints the account instead of writing
            // it, so having run it proves nothing: the configuration is
            // looked up again, and the command fails the ordinary way
            // when nothing landed.
            match Config::from_paths_or_default(config_paths)? {
                Some(config) => config,
                None => bail!(
                    "No configuration found at {}, run `ortie configure` to generate one or write it by hand: {CONFIG_SAMPLE_URL}",
                    path.display(),
                ),
            }
        }
    };

    // NOTE: an empty name and `default` both mean the default account,
    // which is the next block's business.
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
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). Other paths are merged with the first one, which
    /// allows you to separate your public config from your private
    /// one(s). Multiple paths can also be given at once, delimited by
    /// `:` like `$PATH` in a POSIX shell.
    #[arg(long = "config", short = 'c', global = true, env = "ORTIE_CONFIG")]
    #[arg(name = "config_paths", value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub paths: Vec<PathBuf>,
}
