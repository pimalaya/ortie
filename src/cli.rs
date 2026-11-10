//! # Command-line interface
//!
//! The root clap parser of the `ortie` binary, its subcommand router,
//! and the account resolution every account-bound command starts from.

use std::{
    env, error, fmt, fs,
    path::{Path, PathBuf},
};

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
    auth::{
        AuthCommand,
        get::{invocation, shell_quote},
    },
    config::Config,
    json_schema,
    mcp::McpCommand,
    repl::ReplCommand,
    token::TokenCommand,
};

/// The sample configuration: every field, then one section per
/// provider.
const SAMPLE_URL: &str = "https://github.com/pimalaya/ortie/blob/master/config.sample.toml";

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

/// Meets a bare `ortie`: the help when a configuration exists, the
/// missing configuration otherwise, which a person meets as the welcome.
pub fn meet_bare_invocation(config_paths: &[PathBuf]) -> Result<()> {
    // NOTE: a broken file counts as a configuration, its error left to
    // the first command reading it.
    if let Ok(None) = Config::from_paths_or_default(config_paths) {
        let path = Config::target_path(config_paths)?;
        return Err(ConfigMissingError { path }.into());
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
    let Some(mut config) = Config::from_paths_or_default(config_paths)? else {
        let path = Config::target_path(config_paths)?;
        return Err(ConfigMissingError { path }.into());
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

/// No configuration file at the path account resolution looked for.
#[derive(Debug)]
pub struct ConfigMissingError {
    /// Where `-c` pointed, or the default location when it named none,
    /// so a mistyped path shows up as itself.
    pub path: PathBuf,
}

impl fmt::Display for ConfigMissingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.display();
        write!(
            f,
            "No configuration found at {path}, start from the sample: {SAMPLE_URL}"
        )
    }
}

impl error::Error for ConfigMissingError {}

/// Renders the welcome a person meets when no configuration exists at
/// `path`: the commands creating it from `sample`, the shipped sample
/// configuration, or downloading it without one, then the first command
/// to run, each paragraph on one line for the terminal to fold.
pub fn welcome(path: &Path, config_paths: &[PathBuf], sample: Option<&Path>) -> String {
    let file = shell_quote(&path.to_string_lossy());
    let ortie = invocation(config_paths, None);
    let mut create = String::new();

    if let Some(sample) = sample {
        let parent = path.parent().filter(|dir| !dir.as_os_str().is_empty());

        if let Some(dir) = parent.filter(|dir| !dir.is_dir()) {
            let mkdir = if cfg!(windows) { "mkdir" } else { "mkdir -p" };
            let dir = shell_quote(&dir.to_string_lossy());
            create.push_str(&format!("  {mkdir} {dir}\n"));
        }

        // A sample from the Nix store is read-only, which `cp` would keep.
        let copy = if cfg!(windows) {
            "copy"
        } else {
            "install -m 644"
        };
        let sample = shell_quote(&sample.to_string_lossy());
        create.push_str(&format!("  {copy} {sample} {file}\n"));
    } else {
        // In Windows PowerShell 5.1, `curl` is Invoke-WebRequest.
        let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
        let url = SAMPLE_RAW_URL;
        create.push_str(&format!("  {curl} -sSL --create-dirs -o {file} {url}\n"));
    }

    format!(
        "\
Welcome to Ortie, the CLI to manage OAuth tokens.

No configuration found at {path}. Create it from the sample configuration:

{create}
Edit it and uncomment the options you need, then get a first token:

  {ortie} auth get
",
        path = path.display(),
    )
}

/// Finds the sample configuration a release or a package ships next to
/// the running binary.
pub fn shipped_sample() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;

    // A symlinked binary (a Nix profile, Homebrew) ships it beside its
    // target, and a canonical Windows path carries a `\\?\` prefix.
    let exe = if cfg!(windows) {
        exe
    } else {
        fs::canonicalize(exe).ok()?
    };

    find_sample(exe.parent()?)
}

/// Finds share/doc/ortie/config.sample.toml under the prefix holding the
/// `bin` directory of the binary, or under `bin` itself, where an
/// extracted release archive puts it.
fn find_sample(bin: &Path) -> Option<PathBuf> {
    let sample: PathBuf = ["share", "doc", "ortie", "config.sample.toml"]
        .iter()
        .collect();

    [bin.parent(), Some(bin)]
        .into_iter()
        .flatten()
        .map(|prefix| prefix.join(&sample))
        .find(|path| path.is_file())
}

/// Path(s) to the TOML configuration file(s).
#[derive(Debug, Default, Parser)]
pub struct ConfigPathsArg {
    /// Override the default configuration file path.
    ///
    /// Paths are shell-expanded, then canonicalized when applicable.
    /// Repeat the flag to merge several files: the first one is the base
    /// and the next ones are merged on top, which is how a public config
    /// and a private one combine.
    #[arg(long = "config", short = 'c', global = true)]
    #[arg(name = "config_paths", value_name = "PATH", value_parser = path_parser)]
    pub paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_path_is_never_split() {
        let args = ["ortie", "-c", r"C:\a.toml", "-c", "b:c;d.toml"];
        let cli = Cli::try_parse_from(args).unwrap();
        let paths = [PathBuf::from(r"C:\a.toml"), PathBuf::from("b:c;d.toml")];
        assert_eq!(cli.config.paths, paths);
    }

    #[test]
    fn a_missing_configuration_is_typed_for_the_welcome() {
        let paths = [PathBuf::from("/nonexistent/ortie/config.toml")];
        let err = take_account(&paths, None).unwrap_err();
        let missing = err.downcast_ref::<ConfigMissingError>().unwrap();
        assert_eq!(missing.path, paths[0]);
    }

    #[test]
    #[cfg(not(windows))]
    fn the_welcome_commands_run_as_printed() {
        let paths = [PathBuf::from("/a b/it's.toml")];
        let file = r"'/a b/it'\''s.toml'";

        let downloaded = welcome(&paths[0], &paths, None);
        let curl = format!("\n  curl -sSL --create-dirs -o {file} https://");
        assert!(downloaded.contains(&curl));
        assert!(downloaded.contains(&format!("\n  ortie -c {file} auth get\n")));

        let sample = Path::new("/usr/share/doc/ortie/config.sample.toml");
        let copied = welcome(&paths[0], &paths, Some(sample));
        let copy = format!(
            "\n  mkdir -p '/a b'\n  install -m 644 '{}' {file}\n",
            sample.display()
        );
        assert!(copied.contains(&copy));
        assert!(!copied.contains("curl"));
    }

    #[test]
    #[cfg(windows)]
    fn the_welcome_commands_run_in_cmd_and_powershell() {
        let paths = [PathBuf::from(r"C:\a b\c.toml")];
        let welcome = welcome(&paths[0], &paths, None);
        let curl = r#"  curl.exe -sSL --create-dirs -o "C:\a b\c.toml" https://"#;
        assert!(welcome.contains(curl));
        assert!(welcome.contains(r#"  ortie -c "C:\a b\c.toml" auth get"#));
    }

    #[test]
    fn the_shipped_sample_is_found_installed_or_extracted() {
        let prefix = tempfile::tempdir().unwrap();
        let bin = prefix.path().join("bin");
        let sample = prefix.path().join("share/doc/ortie/config.sample.toml");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(sample.parent().unwrap()).unwrap();
        assert_eq!(find_sample(&bin), None);

        fs::write(&sample, "").unwrap();
        assert_eq!(find_sample(&bin), Some(sample.clone()));
        assert_eq!(find_sample(prefix.path()), Some(sample));
    }
}
