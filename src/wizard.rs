//! # Configuration wizard
//!
//! Run on bare `ortie` and by `ortie configure`, walking one prompt at
//! a time to a complete account and printing it as a ready-to-append
//! TOML fragment on stdout.
//!
//! The banner, the prompts and the spinners all render on stderr, so
//! `ortie >> <config>` works as the write-back when stdout is
//! redirected. A terminal is offered the save instead, and an existing
//! file is appended to rather than rewritten, so it stays user-owned.
//!
//! One prompt takes an email address, a bare domain or an issuer URL,
//! and its shape orients the setup, mirroring the Himalaya wizard. An
//! address fans out over io-pim-discovery (see [`search`]); an issuer
//! resolves through its RFC 8414 metadata alone.
//!
//! Every OAuth 2.0 grant that turns up becomes one selectable
//! configuration, tagged with the services sharing it. Discovering
//! nothing stops the wizard on the documented sample rather than
//! dropping into a hand-entry flow.
//!
//! From there the flow narrows the account down: the application
//! backing it (see [`client`]), the scopes that application may request
//! (see [`scope`]), then where its token lives (see [`storage`]).
//!
//! The wizard never runs a grant itself: it hands back a config, and
//! `ortie auth get` is what authorizes it.

pub mod client;
pub mod scope;
pub mod search;
pub mod storage;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{IsTerminal, Write, stdin, stdout},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use pimalaya_cli::{printer::Printer, prompt, spinner::Spinner};
use serde::Serialize;
use url::Url;

use io_pim_discovery::{
    compose::config::DiscoveryAuthMethod, rfc8414::DiscoveryOauthServerMetadata,
};

use pimalaya_config::toml::TomlConfig;

use crate::{config::Config, wizard::search::Discovered};

/// The documented sample configuration, shown in the welcome banner and
/// pointed at whenever discovery finds nothing.
pub const CONFIG_SAMPLE_URL: &str =
    "https://github.com/pimalaya/ortie/blob/master/config.sample.toml";

/// Configure an account interactively.
///
/// Discovers an OAuth 2.0 account from an email address, a bare domain
/// or an issuer URL, prints it, then offers to save it to the
/// configuration file. Anything discovery does not cover is written by
/// hand, every field being documented in the sample configuration.
#[derive(Debug, Parser)]
pub struct ConfigureCommand;

impl ConfigureCommand {
    /// Runs the wizard, then prints the account and offers to save it.
    ///
    /// No welcome, since whoever typed the command knows what it does.
    /// The banner belongs to the offer a missing configuration raises,
    /// where the wizard meets someone who did not ask for it.
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        if !printer.is_json() && !stdin().is_terminal() {
            bail!(
                "Configuring needs a terminal to prompt on, \
                 write the configuration by hand instead: {CONFIG_SAMPLE_URL}"
            );
        }

        run(printer, config_paths)
    }
}

/// Runs the wizard and prints the resulting account as a
/// ready-to-append TOML document on stdout.
///
/// The fragment reaches stdout before the save is offered, so the
/// choice of where it goes is made having seen what is placed.
fn run(printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
    let path = Config::target_path(config_paths)?;
    let existing = ExistingConfig::read(&path)?;

    let input = prompt::text("Email address:", None)?;
    let input = input.trim();
    if input.is_empty() {
        bail!("Empty input: enter an email address, a bare domain, or an issuer URL");
    }

    // NOTE: the account name is only the TOML table key, so it is
    // derived from the input rather than prompted.
    let account_name = account_name(&default_account_name(input), existing.as_ref());
    let mut config = configure_discovery(input)?;
    config.name = account_name;

    // NOTE: a second `default = true` would make the account every
    // command picks depend on map ordering, so the generated one claims
    // the default only when no other account does.
    config.default = !existing.as_ref().is_some_and(|config| config.has_default);

    fill_provider_defaults(&mut config);

    // NOTE: the metadata answers two later steps at once, its
    // registration endpoint deciding whether dynamic registration is on
    // offer and its scopes widening what an unbound client may ask for.
    let metadata = probe_metadata(&config);

    // NOTE: the application comes first, since what a token may request
    // is a property of the application requesting it.
    let scopes = client::configure(&mut config, metadata.as_ref())?;
    scope::prompt(&mut config, scopes)?;
    storage::configure(&mut config)?;

    if !printer.is_json() && config.client_id.is_none() {
        print_missing_application();
    }

    // NOTE: the fragment always reaches stdout, JSON mode and a
    // redirected stdout stopping there so scripts and
    // `ortie >> config.toml` stay non-interactive.
    printer.out(&config)?;

    if printer.is_json() || !stdout().is_terminal() {
        return Ok(());
    }

    offer_save(&config, &path)
}

/// What a configuration already on disk constrains in the generated
/// account: the names it takes, and whether it claims the default.
struct ExistingConfig {
    names: Vec<String>,
    has_default: bool,
}

impl ExistingConfig {
    /// Reads the configuration at `path`, or `None` when no file is
    /// there.
    ///
    /// A file that fails to parse is an error rather than a `None`:
    /// appending to a broken document would bury the actual problem
    /// under a second one.
    fn read(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let config = Config::from_paths(&[path.to_path_buf()])
            .with_context(|| format!("Read the configuration at {}", path.display()))?;

        Ok(Some(Self {
            names: config.accounts.keys().cloned().collect(),
            has_default: config.accounts.values().any(|account| account.default),
        }))
    }
}

/// The name discovery proposes, suffixed until the configuration does
/// not already hold it.
///
/// It has to be free: a second `[accounts.<name>]` table makes the
/// whole document fail to parse, taking down the accounts that used to
/// work with it.
fn account_name(base: &str, existing: Option<&ExistingConfig>) -> String {
    let taken = existing
        .map(|config| config.names.as_slice())
        .unwrap_or(&[]);

    if !taken.iter().any(|name| name == base) {
        return base.to_string();
    }

    let mut suffix = 2;

    loop {
        let name = format!("{base}-{suffix}");

        if !taken.contains(&name) {
            return name;
        }

        suffix += 1;
    }
}

/// Explains, on stderr, the empty `client-id` a custom application
/// leaves behind, right before the fragment it belongs to.
///
/// The wizard stops short of prompting for those fields: registering an
/// application of one's own is the rare path, and whoever took it is
/// already editing the configuration.
fn print_missing_application() {
    eprintln!();
    eprintln!("The wizard stops here. Fill in `client-id` by hand, along with");
    eprintln!("`client-secret.raw` and `endpoints.redirection` if your provider");
    eprintln!("requires them. Every field is documented in the sample configuration:");
    eprintln!();
    eprintln!("  {CONFIG_SAMPLE_URL}");
    eprintln!();
}

/// Frames Ortie, names the configuration file that is missing, and
/// points at the sample for everything the wizard does not cover.
///
/// Printed before the offer a missing configuration raises, so the
/// wizard introduces itself to someone who did not ask for it.
/// `configure` skips it, having been asked for by name.
///
/// On stderr, so a redirected stdout holds the fragment alone.
pub fn print_welcome(path: &Path) {
    eprintln!();
    eprintln!("Welcome to Ortie, the CLI to manage OAuth 2.0 tokens.");
    eprintln!();
    eprintln!("Ortie runs the OAuth 2.0 grant your provider expects and keeps the");
    eprintln!("resulting access token fresh, so any tool that needs one just reads");
    eprintln!("it from your credential manager. It needs one account to work with,");
    eprintln!("and no configuration file was found at:");
    eprintln!();
    eprintln!("  {}", path.display());
    eprintln!();
    eprintln!("The wizard sets that account up for you, from your email address");
    eprintln!("alone. To write it by hand instead, every field is documented at:");
    eprintln!();
    eprintln!("  {CONFIG_SAMPLE_URL}");
    eprintln!();
    eprintln!("At anytime, you can create a new account with the command:");
    eprintln!();
    eprintln!("  ortie configure");
    eprintln!();
}

/// Offers to save the account to a config file, by default
/// $XDG_CONFIG_HOME/ortie/config.toml.
///
/// The account is already printed by then, so the prompt has one
/// meaning and declining leaves the fragment to place by hand. Prompts
/// and confirmations render on stderr.
///
/// An existing file is appended to, never overwritten: the fragment is
/// one `[accounts.<name>]` table, so appending leaves the accounts
/// already configured, and every comment around them, untouched.
///
/// That is what `ortie >> <config>` does, done for the user, and it is
/// confirmed first since the file is one the user already owns.
fn offer_save(config: &OauthConfig, path: &Path) -> Result<()> {
    eprintln!();

    let question = format!("Save this configuration to {}?", path.display());

    if !prompt::bool(question, true)? {
        return Ok(());
    }

    // NOTE: a config rarely ends on a blank line, and two tables glued
    // together read as one, so separate them when appending.
    let appending = fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0);
    let separator = if appending { "\n" } else { "" };

    // NOTE: a file already holding accounts is the user's, so appending
    // is confirmed rather than assumed. Declining loses nothing: the
    // fragment is printed and can be placed by hand.
    if appending {
        let question = format!("{} already exists, append to it?", path.display());

        if !prompt::bool(question, true)? {
            return Ok(());
        }
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Create config directory `{}`", parent.display()))?;
    }

    let mut file = fs::File::options()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Open config file `{}`", path.display()))?;

    write!(file, "{separator}{config}")
        .with_context(|| format!("Write config file `{}`", path.display()))?;

    let verb = if appending { "appended to" } else { "saved to" };
    eprintln!();
    eprintln!("Configuration {verb} {}.", path.display());

    // NOTE: the account is named, the file it landed in likely holding
    // more than this one. One still missing its client id cannot
    // authorize yet, and was told what to fill in already.
    if config.client_id.is_some() {
        eprintln!(
            "Run `ortie auth get --account {}` to authorize the account.",
            config.name
        );
    }

    Ok(())
}

/// Searches the OAuth 2.0 grants reachable from `input`, then folds the
/// one the user picks into a fresh account.
///
/// Discovering nothing stops the wizard rather than prompting for
/// hand-entered endpoints (see [`stop_undiscovered`]).
fn configure_discovery(input: &str) -> Result<OauthConfig> {
    let spinner = Spinner::start("Searching for OAuth 2.0 grants");

    // NOTE: an issuer URL names an authorization server directly, so
    // its metadata is the whole search; anything else is an address,
    // and a bare domain is discovered as `@domain`.
    let mut found = if input.contains("://") {
        search::search_issuer(input)?
    } else if input.contains('@') {
        search::search(input)?
    } else {
        search::search(&format!("@{input}"))?
    };

    if found.is_empty() {
        spinner.failure("No OAuth 2.0 grant found");
        return stop_undiscovered(input);
    }

    spinner.success(format!("Found {} OAuth 2.0 grant(s)", found.len()));

    // NOTE: a lone grant is not a choice; the endpoint spellings a
    // provider is described with are already folded into one entry.
    let choice = match found.len() {
        1 => found.remove(0),
        _ => prompt::item("Choose an OAuth 2.0 grant:", found, None)?,
    };

    Ok(OauthConfig::from(choice))
}

/// Stops the wizard when discovery found nothing for `input`.
///
/// It errors out on the documented sample rather than dropping into a
/// hand-entry flow, the wizard configuring only what it discovers.
fn stop_undiscovered(input: &str) -> Result<OauthConfig> {
    bail!(
        "Could not automatically discover an OAuth 2.0 grant for `{input}`.\n\n\
         Write your account configuration by hand instead, starting from the \
         documented sample:\n  {CONFIG_SAMPLE_URL}"
    )
}

/// Fetches the authorization server metadata behind the chosen grant,
/// behind a spinner since it is a network round trip.
///
/// A server publishing none costs only fewer scope options and no
/// dynamic registration entry.
fn probe_metadata(config: &OauthConfig) -> Option<DiscoveryOauthServerMetadata> {
    let spinner = Spinner::start("Reading the authorization server metadata");

    match search::metadata(&config.endpoints.hosts()) {
        Some(metadata) => {
            spinner.success("Authorization server metadata read");
            Some(metadata)
        }
        None => {
            spinner.failure("No authorization server metadata published");
            None
        }
    }
}

/// Proposes an account name from the input shape: the first label of
/// the domain, or of the issuer host.
fn default_account_name(input: &str) -> String {
    if let Ok(url) = Url::parse(input)
        && let Some(host) = url.host_str()
    {
        return first_label(host);
    }

    match input.rsplit_once('@') {
        Some((_, domain)) => first_label(domain),
        None => first_label(input),
    }
}

/// The first dot-separated label of a host or domain.
fn first_label(host: &str) -> String {
    host.split('.').next().unwrap_or(host).to_string()
}

/// The account resolved by the wizard, printed as a ready-to-append
/// config fragment.
///
/// It renders as bare TOML on stdout, the framing living in the stderr
/// welcome banner, or as the same data in an object under `--json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OauthConfig {
    /// The account name, heading the `[accounts.<name>]` table.
    pub name: String,
    /// Whether this account is picked when none is named, claimed only
    /// when no other account already does.
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    pub default: bool,
    /// The OAuth 2.0 client identifier, when already registered.
    ///
    /// Always serialized, empty included, so both output shapes carry
    /// the placeholder the user fills in by hand.
    pub client_id: Option<String>,
    /// The client secret paired with the identifier, for providers
    /// issuing one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<RawSecret>,
    /// The wire name of the discovered grant flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant: Option<&'static str>,
    /// The discovered endpoints.
    pub endpoints: Endpoints,
    /// The scopes the token will carry.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    /// Extra authorization-request parameters a provider requires but
    /// discovery does not surface; see cairn/changes/discovery-layering/.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extras: BTreeMap<String, String>,
    /// Whether token show refreshes an expired token by itself, which
    /// the wizard always enables.
    pub auto_refresh: bool,
    /// The commands persisting and reading back the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
}

impl OauthConfig {
    /// An account with nothing resolved yet, the base every discovered
    /// grant fills in.
    pub fn empty() -> Self {
        Self {
            default: false,
            name: String::new(),
            client_id: None,
            client_secret: None,
            grant: None,
            endpoints: Endpoints::default(),
            scopes: Vec::new(),
            extras: BTreeMap::new(),
            auto_refresh: true,
            storage: None,
        }
    }
}

impl From<Discovered> for OauthConfig {
    fn from(discovered: Discovered) -> Self {
        match discovered.method {
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint,
                token_endpoint,
                scope,
            } => Self {
                grant: Some("authorization-code"),
                endpoints: Endpoints {
                    authorization: Some(authorization_endpoint),
                    token: Some(token_endpoint),
                    ..Default::default()
                },
                scopes: split_scopes(scope),
                ..Self::empty()
            },
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint,
                token_endpoint,
                scope,
            } => Self {
                grant: Some("device"),
                endpoints: Endpoints {
                    device_authorization: Some(device_authorization_endpoint),
                    token: Some(token_endpoint),
                    ..Default::default()
                },
                scopes: split_scopes(scope),
                ..Self::empty()
            },
            // NOTE: search resolves every issuer into one of the grants
            // above, and drops the non-OAuth methods.
            _ => unreachable!("search yields resolved OAuth grants only"),
        }
    }
}

impl fmt::Display for OauthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[accounts.{}]", toml_key(&self.name))?;

        if self.default {
            writeln!(f, "default = true")?;
        }

        let client_id = self.client_id.as_deref().unwrap_or_default();
        writeln!(f, "client-id = {}", toml_string(client_id))?;

        if let Some(secret) = &self.client_secret {
            writeln!(f, "client-secret.raw = {}", toml_string(&secret.raw))?;
        }

        if let Some(grant) = &self.grant {
            writeln!(f, "grant = {}", toml_string(grant))?;
        }
        if let Some(url) = &self.endpoints.authorization {
            writeln!(f, "endpoints.authorization = {}", toml_string(url))?;
        }
        if let Some(url) = &self.endpoints.device_authorization {
            writeln!(f, "endpoints.device-authorization = {}", toml_string(url))?;
        }
        if let Some(url) = &self.endpoints.token {
            writeln!(f, "endpoints.token = {}", toml_string(url))?;
        }
        if let Some(url) = &self.endpoints.redirection {
            writeln!(f, "endpoints.redirection = {}", toml_string(url))?;
        }

        if !self.scopes.is_empty() {
            writeln!(f, "scopes = {}", toml_array(&self.scopes))?;
        }

        for (key, value) in &self.extras {
            writeln!(f, "extras.{key} = {}", toml_string(value))?;
        }

        writeln!(f, "auto-refresh = {}", self.auto_refresh)?;

        match &self.storage {
            Some(storage) => {
                writeln!(f, "storage.read.command = {}", storage.read.command)?;
                writeln!(f, "storage.write.command = {}", storage.write.command)
            }
            None => {
                writeln!(f, "storage.read.command = \"\"")?;
                writeln!(f, "storage.write.command = \"\"")
            }
        }
    }
}

/// The client secret in the config's `client-secret.raw` shape.
#[derive(Debug, Serialize)]
pub struct RawSecret {
    /// The secret value, stored in clear as the provider issued it.
    pub raw: String,
}

/// Endpoint subset of the account config fragment.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Endpoints {
    /// Authorization endpoint of the authorization code grant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    /// Device authorization endpoint of the device grant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_authorization: Option<String>,
    /// Token endpoint shared by grants and refreshes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Redirection endpoint, when the provider pins one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirection: Option<String>,
}

impl Endpoints {
    /// The distinct lowercased hosts of the endpoints, what the
    /// metadata probe and the known applications key on.
    pub fn hosts(&self) -> BTreeSet<String> {
        let urls = [&self.authorization, &self.device_authorization, &self.token];

        urls.into_iter()
            .flatten()
            .filter_map(|url| Url::parse(url).ok())
            .filter_map(|url| url.host_str().map(str::to_ascii_lowercase))
            .collect()
    }
}

/// Storage subset of the account config fragment.
#[derive(Debug, Serialize)]
pub struct Storage {
    /// The command printing the stored token JSON on its stdout.
    pub read: StorageEntry,
    /// The command receiving the token JSON on its stdin.
    pub write: StorageEntry,
}

/// One direction of the token storage, holding its command.
#[derive(Debug, Serialize)]
pub struct StorageEntry {
    /// The command run for this direction.
    pub command: StorageCommand,
}

/// One storage command, in either shape the config accepts.
///
/// A known credential provider yields an [`Argv`](Self::Argv), the
/// preferred form: no shell sits between Ortie and the program, so
/// nothing in an entry name is reinterpreted.
///
/// Only what genuinely needs shell features falls back to a
/// [`Shell`](Self::Shell) line: the macOS keychain write, where
/// `$(cat)` bridges a secret, and anything typed by hand.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StorageCommand {
    /// A program and its arguments, run with no shell.
    Argv(Vec<String>),
    /// A shell command line, run through the platform shell.
    Shell(String),
}

impl fmt::Display for StorageCommand {
    /// Renders the command as its TOML value.
    ///
    /// An argv becomes an array of basic strings, a shell line a
    /// single-quoted literal, so its own quotes need no escaping.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Argv(argv) => write!(f, "{}", toml_array(argv)),
            Self::Shell(command) => write!(f, "{}", toml_literal(command)),
        }
    }
}

/// Renders `values` as a TOML array of basic strings.
fn toml_array(values: &[String]) -> String {
    let values: Vec<String> = values.iter().map(|value| toml_string(value)).collect();
    format!("[{}]", values.join(", "))
}

/// Renders a TOML basic (double-quoted) string, escaping the two
/// characters that cannot appear raw in one.
fn toml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Renders a TOML literal (single-quoted) string, which escapes
/// nothing, so a shell line keeps its own quoting verbatim.
///
/// A line carrying a single quote cannot be written that way and falls
/// back to a basic string.
fn toml_literal(value: &str) -> String {
    if value.contains('\'') {
        return toml_string(value);
    }

    format!("'{value}'")
}

/// Quotes an account name into a valid TOML table key when it is not a
/// bare key (letters, digits, dashes and underscores only).
fn toml_key(name: &str) -> Cow<'_, str> {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if bare {
        Cow::Borrowed(name)
    } else {
        Cow::Owned(format!("\"{name}\""))
    }
}

/// Splits a space-separated scope string into the config list shape.
pub fn split_scopes(scope: Option<String>) -> Vec<String> {
    scope
        .map(|scope| scope.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default()
}

/// Fills the defaults a provider needs but discovery does not surface.
///
/// Fastmail bounces the flow pre-consent without the RFC 8707 resource
/// indicator, and its discovered grant carries no scopes, so both are
/// supplied. A stopgap; see cairn/changes/discovery-layering/.
fn fill_provider_defaults(config: &mut OauthConfig) {
    let hosts = config.endpoints.hosts();

    if hosts.contains("api.fastmail.com") {
        config
            .extras
            .entry("resource".to_string())
            .or_insert_with(|| "https://api.fastmail.com/jmap/session".to_string());

        if config.scopes.is_empty() {
            config.scopes = advertised_scopes(&config.endpoints)
                .into_iter()
                .map(ToString::to_string)
                .collect();
        }
    }
}

/// The scopes a provider advertises outside its RFC 8414 metadata,
/// folded into the scope options.
///
/// Empty for providers whose scopes discovery or metadata already
/// fills. A stopgap; see cairn/changes/discovery-layering/.
fn advertised_scopes(endpoints: &Endpoints) -> Vec<&'static str> {
    if endpoints.hosts().contains("api.fastmail.com") {
        return vec![
            "urn:ietf:params:oauth:scope:mail",
            "urn:ietf:params:oauth:scope:contacts",
            "urn:ietf:params:oauth:scope:calendars",
            "offline_access",
        ];
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use pimalaya_config::toml::TomlConfig;

    use crate::config::{Config, GrantConfig};

    use super::*;

    /// An address takes the domain's first label, never the local part.
    ///
    /// A bare domain and the synthesized `@domain` form do the same, an
    /// issuer URL takes the host's first label.
    #[test]
    fn account_name_defaults_to_the_first_domain_label() {
        assert_eq!(default_account_name("clement.douin@posteo.net"), "posteo");
        assert_eq!(default_account_name("alice@mail.example.co.uk"), "mail");
        assert_eq!(default_account_name("posteo.net"), "posteo");
        assert_eq!(default_account_name("@posteo.net"), "posteo");
        assert_eq!(
            default_account_name("https://login.microsoftonline.com/common/v2.0"),
            "login"
        );
    }

    #[test]
    fn a_discovered_grant_becomes_its_config_shape() {
        let code = OauthConfig::from(Discovered {
            method: DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: "https://as/auth".to_string(),
                token_endpoint: "https://as/token".to_string(),
                scope: Some("mail offline_access".to_string()),
            },
            services: BTreeSet::new(),
        });

        assert_eq!(code.grant, Some("authorization-code"));
        assert_eq!(
            code.endpoints.authorization.as_deref(),
            Some("https://as/auth")
        );
        assert_eq!(code.endpoints.device_authorization, None);
        assert_eq!(code.scopes, ["mail", "offline_access"]);
        assert!(code.auto_refresh);

        let device = OauthConfig::from(Discovered {
            method: DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: "https://as/device".to_string(),
                token_endpoint: "https://as/token".to_string(),
                scope: None,
            },
            services: BTreeSet::new(),
        });

        assert_eq!(device.grant, Some("device"));
        assert_eq!(device.endpoints.authorization, None);
        assert!(device.scopes.is_empty());
    }

    /// An argv reads back as a TOML array and a shell line as a string,
    /// and a name TOML would read as a path gets quoted.
    #[test]
    fn the_fragment_carries_no_leading_comment() {
        let mut config = OauthConfig {
            default: true,
            name: "posteo".to_string(),
            client_id: Some("client".to_string()),
            grant: Some("authorization-code"),
            endpoints: Endpoints {
                authorization: Some("https://as/auth".to_string()),
                token: Some("https://as/token".to_string()),
                ..Default::default()
            },
            scopes: vec!["mail".to_string()],
            storage: Some(Storage {
                read: StorageEntry {
                    command: StorageCommand::Argv(vec![
                        "pass".to_string(),
                        "show".to_string(),
                        "posteo".to_string(),
                    ]),
                },
                write: StorageEntry {
                    command: StorageCommand::Shell("pass insert -m -f posteo".to_string()),
                },
            }),
            ..OauthConfig::empty()
        };

        let rendered = config.to_string();
        assert!(!rendered.contains('#'), "{rendered}");
        assert!(rendered.starts_with("[accounts.posteo]\n"), "{rendered}");
        assert!(rendered.contains("client-id = \"client\"\n"));
        assert!(rendered.contains("scopes = [\"mail\"]\n"));
        assert!(rendered.contains("auto-refresh = true\n"));

        assert!(rendered.contains("storage.read.command = [\"pass\", \"show\", \"posteo\"]\n"));
        assert!(rendered.contains("storage.write.command = 'pass insert -m -f posteo'\n"));

        config.name = "me@posteo.net".to_string();
        assert!(
            config
                .to_string()
                .starts_with("[accounts.\"me@posteo.net\"]")
        );
    }

    /// A basic string escapes the backslash and the double quote.
    ///
    /// A literal keeps a shell line's own quoting, which is what makes
    /// the macOS keychain write readable, unless the line carries the
    /// one character a literal cannot hold.
    #[test]
    fn rendered_values_survive_the_characters_toml_reserves() {
        assert_eq!(toml_string(r#"a\b"c"#), r#""a\\b\"c""#);
        assert_eq!(
            toml_array(&["one".to_string(), r#"tw"o"#.to_string()]),
            r#"["one", "tw\"o"]"#
        );

        assert_eq!(
            toml_literal(r#"security add-generic-password -w "$(cat)""#),
            r#"'security add-generic-password -w "$(cat)"'"#
        );
        assert_eq!(toml_literal("it's"), r#""it's""#);
    }

    /// The whole point of the fragment: what the wizard prints is what
    /// the config loader accepts, both command shapes included.
    #[test]
    fn a_fragment_parses_back_into_the_account_it_came_from() {
        let mut config = OauthConfig {
            name: "posteo".to_string(),
            client_id: Some("client".to_string()),
            grant: Some("authorization-code"),
            endpoints: Endpoints {
                authorization: Some("https://as/auth".to_string()),
                token: Some("https://as/token".to_string()),
                ..Default::default()
            },
            scopes: vec!["mail".to_string(), "offline_access".to_string()],
            storage: Some(Storage {
                read: StorageEntry {
                    command: StorageCommand::Argv(vec![
                        "secret-tool".to_string(),
                        "lookup".to_string(),
                        "account".to_string(),
                        "posteo".to_string(),
                    ]),
                },
                write: StorageEntry {
                    command: StorageCommand::Shell(
                        r#"security add-generic-password -U -a posteo -w "$(cat)""#.to_string(),
                    ),
                },
            }),
            ..OauthConfig::empty()
        };
        config.extras.insert(
            "resource".to_string(),
            "https://api.fastmail.com/jmap/session".to_string(),
        );

        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "{config}").unwrap();

        let account = Config::from_paths(&[file.path().to_path_buf()])
            .unwrap()
            .take_named_account("posteo")
            .unwrap()
            .1;

        assert_eq!(account.client_id, "client");
        assert_eq!(account.grant, GrantConfig::AuthorizationCode);
        assert_eq!(account.scopes, ["mail", "offline_access"]);
        assert_eq!(
            account.extras.get("resource").map(String::as_str),
            Some("https://api.fastmail.com/jmap/session")
        );
        assert_eq!(
            account.endpoints.token.unwrap().as_str(),
            "https://as/token"
        );
        assert!(account.auto_refresh);
    }

    #[test]
    fn fastmail_gets_its_resource_indicator_and_scopes() {
        let mut config = OauthConfig {
            endpoints: Endpoints {
                token: Some("https://api.fastmail.com/oauth/refresh".to_string()),
                ..Default::default()
            },
            ..OauthConfig::empty()
        };

        fill_provider_defaults(&mut config);

        assert_eq!(
            config.extras.get("resource").map(String::as_str),
            Some("https://api.fastmail.com/jmap/session")
        );
        assert!(config.scopes.contains(&"offline_access".to_string()));
    }

    #[test]
    fn other_providers_get_no_quirk() {
        let mut config = OauthConfig {
            endpoints: Endpoints {
                token: Some("https://as.example.test/token".to_string()),
                ..Default::default()
            },
            ..OauthConfig::empty()
        };

        fill_provider_defaults(&mut config);

        assert!(config.extras.is_empty());
        assert!(config.scopes.is_empty());
    }
}

#[cfg(test)]
mod frame_tests {
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    static NEXT_CONFIG: AtomicUsize = AtomicUsize::new(0);

    /// A path in the temporary directory no other test writes to.
    fn config_path() -> PathBuf {
        let id = NEXT_CONFIG.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("ortie-configure-{id}.toml"))
    }

    #[test]
    fn a_taken_name_gets_a_suffix() {
        let existing = ExistingConfig {
            names: vec!["posteo".to_string(), "posteo-2".to_string()],
            has_default: true,
        };

        assert_eq!(account_name("posteo", None), "posteo");
        assert_eq!(account_name("posteo", Some(&existing)), "posteo-3");
        assert_eq!(account_name("gmail", Some(&existing)), "gmail");
    }

    #[test]
    fn a_missing_configuration_constrains_nothing() {
        let existing = ExistingConfig::read(&config_path()).expect("read a missing config");

        assert!(existing.is_none());
    }

    #[test]
    fn an_existing_configuration_reports_its_names_and_default() {
        let path = config_path();
        fs::write(
            &path,
            "# my accounts\n[accounts.work]\ndefault = true\nclient-id = \"a\"\nstorage.read.command = [\"true\"]\nstorage.write.command = \"true\"\n",
        )
        .expect("write the existing config");

        let existing = ExistingConfig::read(&path)
            .expect("read the existing config")
            .expect("an existing config");

        assert_eq!(existing.names, ["work"]);
        assert!(existing.has_default);

        fs::remove_file(&path).expect("remove the config");
    }

    /// The existing file ends on no trailing newline, the shape an
    /// appended block has to survive without merging into the last line.
    ///
    /// The account it already held keeps its default and its comments.
    #[test]
    fn an_appended_account_keeps_the_existing_one() {
        let path = config_path();

        fs::write(
            &path,
            "# my accounts\n[accounts.work]\ndefault = true\nclient-id = \"a\"\nstorage.read.command = [\"true\"]\nstorage.write.command = \"true\"",
        )
        .expect("write the existing config");

        let existing = ExistingConfig::read(&path)
            .expect("read the existing config")
            .expect("an existing config");

        let mut config = OauthConfig::empty();
        config.name = account_name("work", Some(&existing));
        config.default = !existing.has_default;
        config.client_id = Some("b".to_string());
        config.storage = Some(Storage {
            read: StorageEntry {
                command: StorageCommand::Argv(vec!["true".to_string()]),
            },
            write: StorageEntry {
                command: StorageCommand::Shell("true".to_string()),
            },
        });

        let mut file = fs::File::options()
            .append(true)
            .open(&path)
            .expect("open the config");
        write!(file, "\n{config}").expect("append the generated account");
        drop(file);

        let content = fs::read_to_string(&path).expect("read back");
        let parsed =
            Config::from_paths(std::slice::from_ref(&path)).expect("parse the appended config");

        assert_eq!(parsed.accounts.len(), 2);
        assert!(parsed.accounts.contains_key("work-2"));

        let defaults = parsed
            .accounts
            .values()
            .filter(|account| account.default)
            .count();
        assert_eq!(defaults, 1);
        assert!(parsed.accounts["work"].default);
        assert!(content.starts_with("# my accounts"));

        fs::remove_file(&path).expect("remove the config");
    }
}
