//! Configuration wizard.
//!
//! Run on bare `ortie` (no subcommand), the natural first contact with
//! the tool. It opens with a welcome banner on stderr, then walks one
//! prompt at a time to a complete account, which it prints as a
//! ready-to-append TOML fragment on stdout before offering (when
//! writing to a terminal) to append it to a config file. The banner,
//! the prompts and the spinners all render on stderr, so
//! `ortie >> <config>` still works as the write-back when stdout is
//! redirected, and the config stays user-owned either way: an existing
//! file is appended to, never rewritten.
//!
//! One prompt takes an email address, a bare domain, or an issuer URL,
//! and its shape orients the setup, mirroring the Himalaya wizard:
//!
//! - an email (or bare domain) runs io-pim-discovery's parallel
//!   discovery (see [`search`]) and every OAuth 2.0 grant it advertises
//!   becomes one selectable configuration, tagged with the services
//!   sharing it;
//! - an issuer URL resolves that authorization server's RFC 8414
//!   metadata into the grant it advertises.
//!
//! The wizard only configures what it can discover automatically. When
//! discovery finds nothing for the given input it stops and points at
//! the documented sample, rather than prompting for hand-entered
//! endpoints.
//!
//! From there the flow narrows the account down: the application
//! backing it (see [`client`]), the scopes that application may
//! request (see [`scope`]), then where its token lives (see
//! [`storage`]). The wizard never runs a grant itself: it hands back a
//! config, and `ortie auth get` is what authorizes it.

pub mod client;
pub mod scope;
pub mod search;
pub mod storage;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{IsTerminal, Write, stdout},
    path::Path,
};

use anyhow::{Context, Result, bail};
use pimalaya_cli::{printer::Printer, prompt, spinner::Spinner};
use serde::Serialize;
use url::Url;

use io_pim_discovery::{
    compose::config::DiscoveryAuthMethod, rfc8414::DiscoveryOauthServerMetadata,
};

use crate::wizard::search::Discovered;

/// The documented sample configuration, shown in the welcome banner
/// and pointed at when discovery finds nothing to configure
/// automatically.
const CONFIG_SAMPLE_URL: &str = "https://github.com/pimalaya/ortie/blob/master/config.sample.toml";

/// Runs the wizard and prints the resulting account as a
/// ready-to-append TOML document on stdout.
///
/// A welcome message renders on stderr first (skipped in JSON mode) to
/// frame what Ortie is and what the wizard does, so nothing but the
/// fragment lands on stdout.
pub fn run(printer: &mut impl Printer) -> Result<()> {
    if !printer.is_json() {
        print_welcome();
    }

    let input = prompt::text("Email address:", None)?;
    let input = input.trim();
    if input.is_empty() {
        bail!("Empty input: enter an email address, a bare domain, or an issuer URL");
    }

    // NOTE: the account name is just the TOML table key, so it is
    // derived from the input rather than prompted; the user renames it
    // by hand.
    let account_name = default_account_name(input);
    let mut config = configure_discovery(input)?;
    config.name = account_name;

    // NOTE: fill the defaults a provider is known to need but discovery
    // does not yet surface (Fastmail's RFC 8707 resource and its
    // scopes). Stopgap; see cairn/changes/discovery-layering/.
    fill_provider_defaults(&mut config);

    // The authorization server metadata answers two later steps at
    // once, so probe it once here: its registration endpoint decides
    // whether dynamic registration is on offer, and the scopes it
    // supports widen the options of a client not bound to a registered
    // set.
    let metadata = probe_metadata(&config);

    // NOTE: application first: what a token may request is a property
    // of the application requesting it.
    let scopes = client::configure(&mut config, metadata.as_ref())?;
    scope::prompt(&mut config, scopes)?;
    storage::configure(&mut config)?;

    // The account is complete but for the application, so say what is
    // missing before the fragment carrying the hole.
    if !printer.is_json() && config.client_id.is_none() {
        print_missing_application();
    }

    // The fragment is what the wizard owes the user, so it always
    // reaches stdout. JSON mode and a redirected stdout stop there,
    // staying non-interactive for scripts and `ortie >> config.toml`;
    // a terminal is then offered the save.
    printer.out(&config)?;

    if printer.is_json() || !stdout().is_terminal() {
        return Ok(());
    }

    offer_save(&config)
}

/// Explains, on stderr, the empty `client-id` a custom application
/// leaves behind, right before the fragment it belongs to.
///
/// The wizard stops short of prompting for those fields. Registering
/// an application of one's own is the rare path, and whoever took it
/// is already editing the configuration; typing them into a wizard to
/// check them in a file afterwards helps nobody.
fn print_missing_application() {
    eprintln!();
    eprintln!("The wizard stops here. Fill in `client-id` by hand, along with");
    eprintln!("`client-secret.raw` and `endpoints.redirection` if your provider");
    eprintln!("requires them. Every field is documented in the sample configuration:");
    eprintln!();
    eprintln!("  {CONFIG_SAMPLE_URL}");
    eprintln!();
}

/// Prints a welcome banner on stderr framing the project and the
/// wizard, so bare `ortie` explains itself before dropping into
/// prompts. On stderr so it never pollutes a redirected fragment.
fn print_welcome() {
    eprintln!();
    eprintln!("Welcome to Ortie, the CLI to manage OAuth 2.0 tokens.");
    eprintln!();
    eprintln!("Ortie runs the OAuth 2.0 grant your provider expects and keeps the");
    eprintln!("resulting access token fresh, so any tool that needs one just reads");
    eprintln!("it from your credential manager. It needs one account to work with.");
    eprintln!();
    eprintln!("This wizard sets that account up for you, from your email address");
    eprintln!("alone. To write it by hand instead, every field is documented in the");
    eprintln!("sample configuration:");
    eprintln!();
    eprintln!("  {CONFIG_SAMPLE_URL}");
    eprintln!();
}

/// Offers to save the account to a config file (default
/// `$XDG_CONFIG_HOME/ortie/config.toml`). It has already been printed
/// by then, so the prompt has one meaning and declining simply leaves
/// the user with the fragment to place themselves. Prompts and
/// confirmations render on stderr.
///
/// An existing file is appended to, never overwritten: the fragment is
/// one `[accounts.<name>]` table, so appending adds an account and
/// leaves the ones already configured (and every comment around them)
/// untouched. That is the same thing `ortie >> <config>` does, done
/// for the user, and it is confirmed before it happens since the file
/// is one the user already owns.
fn offer_save(config: &OauthConfig) -> Result<()> {
    eprintln!();

    if !prompt::bool("Save this configuration to a file?", true)? {
        return Ok(());
    }

    let default = default_config_path();
    let path = prompt::text("Configuration file path:", default.as_deref())?;
    let path = shellexpand::full(path.trim())?.into_owned();
    let path = Path::new(&path);

    // NOTE: a config rarely ends on a blank line, and two tables glued
    // together read as one, so separate them when appending.
    let appending = fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0);
    let separator = if appending { "\n" } else { "" };

    // A file already holding accounts is the user's, so appending to it
    // is confirmed rather than assumed. Declining stops the save: the
    // fragment is printed, so nothing is lost by placing it by hand.
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

    // NOTE: name the account, since the file it landed in likely holds
    // more than this one. An account still missing its client id
    // cannot authorize yet, and was told what to fill in already.
    if config.client_id.is_some() {
        eprintln!(
            "Run `ortie auth get --account {}` to authorize the account.",
            config.name
        );
    }

    Ok(())
}

/// The default config path (`$XDG_CONFIG_HOME/ortie/config.toml`),
/// used to seed the save prompt; `None` when no config dir resolves.
fn default_config_path() -> Option<String> {
    let path = dirs::config_dir()?
        .join(env!("CARGO_PKG_NAME"))
        .join("config.toml");

    Some(path.to_string_lossy().into_owned())
}

/// Runs the discovery flow for an email, a bare domain, or an issuer
/// URL: search the OAuth 2.0 grants reachable from it, let the user
/// pick one, then fold it into a fresh account. When nothing is
/// discovered the wizard stops rather than prompting for hand-entered
/// endpoints (see [`stop_undiscovered`]).
fn configure_discovery(input: &str) -> Result<OauthConfig> {
    let spinner = Spinner::start("Searching for OAuth 2.0 grants");

    // An issuer URL names an authorization server directly, so its
    // metadata is the whole search; anything else is an address, and a
    // bare domain is discovered as `@domain`.
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

/// Stops the wizard when discovery found nothing to configure for
/// `input`: it prints where to go next (a hand-written config, seeded
/// from the documented sample) and errors out, rather than dropping
/// into a hand-entry flow. Ortie's wizard only ever configures what it
/// can discover automatically.
fn stop_undiscovered(input: &str) -> Result<OauthConfig> {
    bail!(
        "Could not automatically discover an OAuth 2.0 grant for `{input}`.\n\n\
         Write your account configuration by hand instead, starting from the \
         documented sample:\n  {CONFIG_SAMPLE_URL}"
    )
}

/// Fetches the authorization server metadata behind the chosen grant,
/// behind a spinner since it is a network round trip. Absent when the
/// server publishes none, which only means fewer scope options and no
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
/// the domain (of an email or bare domain) or of the issuer host.
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
/// config fragment: bare TOML on stdout (the framing lives in the
/// stderr welcome banner), or the same data as an object in JSON mode.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OauthConfig {
    /// The account name, heading the `[accounts.<name>]` table.
    pub name: String,
    /// The OAuth 2.0 client identifier, when already registered.
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
    /// Extra authorization-request parameters a provider is known to
    /// require but discovery does not yet surface (Fastmail's RFC 8707
    /// resource). Stopgap; see cairn/changes/discovery-layering/.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extras: BTreeMap<String, String>,
    /// Whether token show refreshes an expired token by itself; the
    /// wizard always enables it.
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
            // NOTE: search resolves every issuer into one of the two
            // grants above, and drops the non-OAuth methods.
            _ => unreachable!("search yields resolved OAuth grants only"),
        }
    }
}

impl fmt::Display for OauthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[accounts.{}]", toml_key(&self.name))?;

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

/// The client secret in the config's secret shape
/// (`client-secret.raw`).
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
    /// The distinct hosts of the endpoints, lowercased: what the
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
/// nothing in an entry name can be reinterpreted. Only the commands
/// that genuinely need shell features fall back to a
/// [`Shell`](Self::Shell) line, the write half of the macOS keychain
/// pair (`$(cat)` bridges a secret the program takes as an argument)
/// and anything the user typed by hand.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StorageCommand {
    /// A program and its arguments, run with no shell.
    Argv(Vec<String>),
    /// A shell command line, run through the platform shell.
    Shell(String),
}

impl fmt::Display for StorageCommand {
    /// Renders the command as its TOML value: an array of basic
    /// strings for an argv, and a literal (single-quoted) string for a
    /// shell line, so the quotes such a line usually carries need no
    /// escaping.
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
/// nothing, so a shell line keeps its own quoting verbatim. A line
/// carrying a single quote cannot be written that way and falls back
/// to a basic string.
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

/// Fills the defaults a provider is known to need but discovery does
/// not yet surface. Fastmail's authorization endpoint bounces the flow
/// pre-consent (no password or scope screen, a straight redirect to the
/// "close this window" page) unless the RFC 8707 resource indicator is
/// present, and its discovered grant carries no scopes at all; supply
/// the resource and, since Fastmail cannot complete on a desktop
/// anyway, its full advertised scope set. Stopgap until discovery
/// surfaces them; see cairn/changes/discovery-layering/.
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

/// The scopes a provider is known to advertise outside its RFC 8414
/// metadata, folded into the scope options. Empty for providers whose
/// scopes discovery or metadata already fills. Stopgap; see
/// cairn/changes/discovery-layering/.
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

    #[test]
    fn account_name_defaults_to_the_first_domain_label() {
        // Email: the domain's first label, never the local part.
        assert_eq!(default_account_name("clement.douin@posteo.net"), "posteo");
        assert_eq!(default_account_name("alice@mail.example.co.uk"), "mail");
        // Bare domain, and the synthesized form discovery uses.
        assert_eq!(default_account_name("posteo.net"), "posteo");
        assert_eq!(default_account_name("@posteo.net"), "posteo");
        // Issuer URL: the host's first label.
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

    #[test]
    fn the_fragment_carries_no_leading_comment() {
        let mut config = OauthConfig {
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

        // An argv reads back as a TOML array, a shell line as a string.
        assert!(rendered.contains("storage.read.command = [\"pass\", \"show\", \"posteo\"]\n"));
        assert!(rendered.contains("storage.write.command = 'pass insert -m -f posteo'\n"));

        // A name TOML would read as a path gets quoted.
        config.name = "me@posteo.net".to_string();
        assert!(
            config
                .to_string()
                .starts_with("[accounts.\"me@posteo.net\"]")
        );
    }

    #[test]
    fn rendered_values_survive_the_characters_toml_reserves() {
        // A basic string escapes the backslash and the double quote.
        assert_eq!(toml_string(r#"a\b"c"#), r#""a\\b\"c""#);
        assert_eq!(
            toml_array(&["one".to_string(), r#"tw"o"#.to_string()]),
            r#"["one", "tw\"o"]"#
        );

        // A shell line keeps its own quoting through a literal string,
        // which is why the macOS keychain write stays readable.
        assert_eq!(
            toml_literal(r#"security add-generic-password -w "$(cat)""#),
            r#"'security add-generic-password -w "$(cat)"'"#
        );

        // Unless it carries the one character a literal cannot hold.
        assert_eq!(toml_literal("it's"), r#""it's""#);
    }

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

        // The whole point of the fragment: what the wizard prints is
        // what the config loader accepts, both command shapes included.
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
