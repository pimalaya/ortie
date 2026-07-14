//! `auth discover` subcommand: interactive OAuth 2.0 account wizard,
//! also run by bare `ortie`.
//!
//! Prompts for an email address, a server or an issuer URI, discovers
//! the PIM services reachable for it through io-pim-discovery, keeps
//! only the ones advertising an OAuth 2.0 method, and lets the user
//! pick one. A trailing manual entry falls back to typing the OAuth
//! 2.0 endpoints by hand. The client step proposes well-known public
//! applications registered against the discovered provider
//! (Thunderbird today); the trailing custom entry prompts for
//! everything a manually registered application needs (client id and
//! secret, scopes, redirection endpoint). The storage step plugs the
//! token into a credential provider CLI known for the running OS,
//! falling back to custom shell commands. The outcome is a complete
//! account config fragment printed as valid TOML on stdout (guidance
//! embedded as comments, prompts on stderr), so `ortie >> <config>`
//! appends it.

use std::{borrow::Cow, collections::BTreeSet, fmt};

use anyhow::{Result, bail};
use clap::Parser;
use log::debug;
use pimalaya_cli::{printer::Printer, prompt, spinner::Spinner};
use pimalaya_stream::tls::{Rustls, Tls};
use serde::Serialize;
use url::Url;

use io_pim_discovery::{
    compose::{
        client::ComposeClientStd,
        types::{AuthMethod, Service, ServiceConfig},
    },
    shared::dns::system_resolver,
};

/// Fallback DNS resolver when the system one cannot be determined.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Discover OAuth 2.0 services and print a ready-to-append account
/// config fragment. This is also what bare `ortie` runs.
///
/// The wizard prompts for an email address, a server or an issuer
/// URI, discovers the reachable services, lets you pick one (or enter
/// the endpoints manually), then prints the resulting account as
/// valid TOML on stdout. It never writes any file: append the
/// fragment to your config yourself, e.g. `ortie >> <config>`.
#[derive(Debug, Parser)]
pub struct AuthDiscoverCommand {
    /// Email address, server or issuer URI to discover OAuth 2.0
    /// services for. Prompted interactively when omitted.
    pub input: Option<String>,
}

impl AuthDiscoverCommand {
    /// Runs the wizard: discover, pick, name, then print the account
    /// config fragment.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let input = match self.input {
            Some(input) => input,
            None => prompt::text::<&str>("Email, server or URI:", None)?,
        };
        let input = input.trim();

        if input.is_empty() {
            bail!("Empty input: enter an email address, a server or an issuer URI");
        }

        // A server or an issuer carries a URI scheme and has nothing
        // to discover, so it goes straight to manual entry. Anything
        // else is an email address; a bare domain is discovered as
        // `@domain`.
        let mut config = if input.contains("://") {
            manual(Some(input))?
        } else {
            let email = if input.contains('@') {
                Cow::Borrowed(input)
            } else {
                Cow::Owned(format!("@{input}"))
            };

            choose(&email)?
        };

        // NOTE: suggest the first label of the input's domain (or
        // URI host) as account name.
        let domain = if let Some((_, domain)) = input.rsplit_once('@') {
            Some(domain.to_string())
        } else if input.contains("://") {
            input
                .parse::<Url>()
                .ok()
                .and_then(|url| url.host_str().map(ToString::to_string))
        } else {
            Some(input.to_string())
        };

        let suggested_name = domain
            .as_deref()
            .and_then(|domain| domain.split('.').next())
            .filter(|label| !label.is_empty())
            .map(ToString::to_string);

        let name = prompt::text("Account name:", suggested_name.as_deref())?;
        let name = name.trim();
        if name.is_empty() {
            bail!("Empty account name");
        }
        config.name = name.to_string();

        // NOTE: well-known public applications registered against the
        // same authorization server can be reused instead of
        // registering a client.
        let apps = known_apps(&config);

        if apps.is_empty() {
            custom_client(&mut config)?;
        } else {
            let mut choices: Vec<ClientChoice> =
                apps.into_iter().map(ClientChoice::Known).collect();
            choices.push(ClientChoice::Custom);

            match prompt::item("Public application:", choices, None)? {
                ClientChoice::Known(app) => {
                    config.client_id = Some(app.client_id.to_string());
                    config.client_secret = app.client_secret.map(|raw| RawSecret {
                        raw: raw.to_string(),
                    });
                    config.endpoints.redirection = app.redirection.map(ToString::to_string);

                    // NOTE: discovered scopes stay, they are narrower
                    // (per service); the app's registered set only
                    // fills the gap when discovery yielded none.
                    if config.scopes.is_empty() {
                        config.scopes = app.scopes.iter().map(ToString::to_string).collect();
                    }
                }
                ClientChoice::Custom => custom_client(&mut config)?,
            }
        }

        // NOTE: plug the token storage into a credential provider
        // CLI known for the running OS, or take custom commands.
        let providers = KnownStorage::available();

        if providers.is_empty() {
            custom_storage(&mut config)?;
        } else {
            let mut choices: Vec<StorageChoice> =
                providers.into_iter().map(StorageChoice::Known).collect();
            choices.push(StorageChoice::Custom);

            match prompt::item("Token storage:", choices, None)? {
                StorageChoice::Known(provider) => {
                    config.storage = Some(Storage {
                        read: StorageEntry {
                            command: provider.read(&config.name),
                        },
                        write: StorageEntry {
                            command: provider.write(&config.name),
                        },
                    });
                }
                StorageChoice::Custom => custom_storage(&mut config)?,
            }
        }

        printer.out(config)
    }
}

/// Runs discovery for `email`, then prompts the user to pick one of
/// the OAuth 2.0 services found (or the trailing manual entry). Falls
/// straight through to manual entry when nothing is found.
fn choose(email: &str) -> Result<OauthConfig> {
    let spinner = Spinner::start("Searching for OAuth 2.0 services");
    let discovered = discover(email)?;

    if discovered.is_empty() {
        spinner.failure("No OAuth 2.0 service found, entering manually");
        return manual(None);
    }

    spinner.success(format!("Found {} OAuth 2.0 service(s)", discovered.len()));

    let mut choices: Vec<Choice> = discovered.into_iter().map(Choice::Discovered).collect();
    choices.push(Choice::Manual);

    match prompt::item("Choose an OAuth 2.0 service:", choices, None)? {
        Choice::Discovered(service) => Ok(service.into_config()),
        Choice::Manual => manual(None),
    }
}

/// Composes service configs for `email` across every discovery
/// mechanism and reduces the result to the deduplicated OAuth 2.0
/// methods it advertises, each tagged with the services that share it.
fn discover(email: &str) -> Result<Vec<DiscoveredOauth>> {
    let resolver = system_resolver().unwrap_or_else(|| {
        DEFAULT_RESOLVER
            .parse()
            .expect("default resolver must be a valid URL")
    });

    let tls = Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };

    let client = ComposeClientStd::new(resolver, tls);

    // The OAuth-capable PIM services; POP3, WebDAV and ManageSieve
    // never advertise an OAuth flow of their own.
    let services = BTreeSet::from([
        Service::Imap,
        Service::Smtp,
        Service::Jmap,
        Service::Caldav,
        Service::Carddav,
    ]);

    debug!("compose OAuth 2.0 services for {email}");
    let configs = client.compose_all(email, services)?;

    Ok(collect_oauth(&configs))
}

/// Collects the OAuth 2.0 methods across every discovered config,
/// deduplicated by method, each carrying the set of services it
/// authenticates.
fn collect_oauth(configs: &[ServiceConfig]) -> Vec<DiscoveredOauth> {
    let mut discovered: Vec<DiscoveredOauth> = Vec::new();

    for config in configs {
        for method in &config.auth {
            if !is_oauth(method) {
                continue;
            }

            match discovered.iter_mut().find(|d| &d.method == method) {
                Some(existing) => {
                    existing.services.insert(config.service);
                }
                None => discovered.push(DiscoveredOauth {
                    method: method.clone(),
                    services: BTreeSet::from([config.service]),
                }),
            }
        }
    }

    discovered
}

/// Whether an authentication method is one of the OAuth 2.0 flows.
fn is_oauth(method: &AuthMethod) -> bool {
    matches!(
        method,
        AuthMethod::OauthAuthorizationCodeGrant { .. }
            | AuthMethod::OauthDeviceAuthorizationGrant { .. }
            | AuthMethod::OauthIssuer(_)
    )
}

/// Prompts for the OAuth 2.0 endpoints by hand. `issuer` pre-seeds
/// the issuer when the input was a bare URI.
fn manual(issuer: Option<&str>) -> Result<OauthConfig> {
    let authorization = prompt::text::<&str>("Authorization endpoint:", None)?;
    let token = prompt::text::<&str>("Token endpoint:", None)?;

    Ok(OauthConfig {
        name: String::new(),
        client_id: None,
        client_secret: None,
        grant: Some("authorization-code"),
        endpoints: Endpoints {
            authorization: Some(authorization),
            device_authorization: None,
            token: Some(token),
            redirection: None,
        },
        scopes: Vec::new(),
        auto_refresh: true,
        issuer: issuer.map(ToString::to_string),
        storage: None,
    })
}

/// Prompts for the details of a manually registered application:
/// client id and secret, granted scopes and the registered
/// redirection endpoint. Everything past the id is skipped when the
/// id is left empty for later.
fn custom_client(config: &mut OauthConfig) -> Result<()> {
    config.client_id = prompt::some_text::<&str>("Client id (leave empty for now):", None)?
        .filter(|id| !id.is_empty());

    if config.client_id.is_none() {
        return Ok(());
    }

    config.client_secret = prompt::some_text::<&str>("Client secret (leave empty if none):", None)?
        .filter(|secret| !secret.is_empty())
        .map(|raw| RawSecret { raw });

    let discovered_scopes = config.scopes.join(" ");
    let default_scopes = (!discovered_scopes.is_empty()).then_some(discovered_scopes.as_str());
    let scopes = prompt::some_text("Scopes (space separated, optional):", default_scopes)?;
    config.scopes = split_scopes(scopes.filter(|scopes| !scopes.is_empty()));

    config.endpoints.redirection =
        prompt::some_text::<&str>("Redirection endpoint (leave empty for default):", None)?
            .filter(|url| !url.is_empty());

    Ok(())
}

/// Prompts for the custom storage commands, run through the platform
/// shell. The write prompt is skipped when the read command is left
/// empty for later, and the fragment keeps empty placeholders.
fn custom_storage(config: &mut OauthConfig) -> Result<()> {
    let read = prompt::some_text::<&str>("Read command (leave empty for now):", None)?
        .filter(|command| !command.is_empty());

    let Some(read) = read else {
        return Ok(());
    };

    let write = prompt::text::<&str>("Write command (receives the token on stdin):", None)?;

    config.storage = Some(Storage {
        read: StorageEntry {
            command: StorageCommand::Shell(read),
        },
        write: StorageEntry {
            command: StorageCommand::Shell(write),
        },
    });

    Ok(())
}

/// One deduplicated OAuth 2.0 method and the services sharing it.
#[derive(Debug, Eq, PartialEq)]
struct DiscoveredOauth {
    method: AuthMethod,
    services: BTreeSet<Service>,
}

impl DiscoveredOauth {
    fn into_config(self) -> OauthConfig {
        match self.method {
            AuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint,
                token_endpoint,
                scope,
            } => OauthConfig {
                name: String::new(),
                client_id: None,
                client_secret: None,
                grant: Some("authorization-code"),
                endpoints: Endpoints {
                    authorization: Some(authorization_endpoint),
                    device_authorization: None,
                    token: Some(token_endpoint),
                    redirection: None,
                },
                scopes: split_scopes(scope),
                auto_refresh: true,
                issuer: None,
                storage: None,
            },
            AuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint,
                token_endpoint,
                scope,
            } => OauthConfig {
                name: String::new(),
                client_id: None,
                client_secret: None,
                grant: Some("device"),
                endpoints: Endpoints {
                    authorization: None,
                    device_authorization: Some(device_authorization_endpoint),
                    token: Some(token_endpoint),
                    redirection: None,
                },
                scopes: split_scopes(scope),
                auto_refresh: true,
                issuer: None,
                storage: None,
            },
            AuthMethod::OauthIssuer(issuer) => OauthConfig {
                name: String::new(),
                client_id: None,
                client_secret: None,
                grant: None,
                endpoints: Endpoints::default(),
                scopes: Vec::new(),
                auto_refresh: true,
                issuer: Some(issuer),
                storage: None,
            },
            // NOTE: collect_oauth only keeps the OAuth variants above.
            _ => unreachable!("collect_oauth retains OAuth methods only"),
        }
    }
}

/// One entry in the service pick list: a discovered service, or the
/// trailing manual entry.
#[derive(Debug, Eq, PartialEq)]
enum Choice {
    Discovered(DiscoveredOauth),
    Manual,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let discovered = match self {
            Self::Discovered(discovered) => discovered,
            Self::Manual => return write!(f, "Enter OAuth 2.0 details manually"),
        };

        let services = discovered
            .services
            .iter()
            .map(|service| service_name(*service))
            .collect::<Vec<_>>()
            .join(", ");

        match &discovered.method {
            AuthMethod::OauthAuthorizationCodeGrant { token_endpoint, .. } => {
                write!(f, "OAuth 2.0 authorization code grant")?;
                write!(f, " ({services}) via {token_endpoint}")
            }
            AuthMethod::OauthDeviceAuthorizationGrant { token_endpoint, .. } => {
                write!(f, "OAuth 2.0 device authorization grant")?;
                write!(f, " ({services}) via {token_endpoint}")
            }
            AuthMethod::OauthIssuer(issuer) => {
                write!(f, "OAuth 2.0 issuer {issuer} ({services})")
            }
            _ => Ok(()),
        }
    }
}

/// One entry in the public application pick list: a well-known
/// public application, or the trailing custom entry.
#[derive(Debug, Eq, PartialEq)]
enum ClientChoice {
    Known(&'static KnownApp),
    Custom,
}

impl fmt::Display for ClientChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(app) => write!(f, "{} ({})", app.name, app.covers),
            Self::Custom => write!(f, "Custom application"),
        }
    }
}

/// A well-known public application whose client id is reusable.
///
/// Providers bind a registration to their own authorization server,
/// so each entry carries the endpoint host it was registered against
/// and only shows up when the account's endpoints live on that host.
#[derive(Debug, Eq, PartialEq)]
struct KnownApp {
    /// Display name of the application.
    name: &'static str,
    /// The PIM domains the registration covers, shown between parens
    /// in the pick list; derived from the scopes the application is
    /// registered for.
    covers: &'static str,
    /// Host of the endpoints the client is registered against.
    host: &'static str,
    /// The public client identifier.
    client_id: &'static str,
    /// The client secret, for providers issuing one; as public as the
    /// client id.
    client_secret: Option<&'static str>,
    /// Redirect URI registered with the provider, when it must be
    /// pinned; the runtime default (http://127.0.0.1:0) otherwise.
    redirection: Option<&'static str>,
    /// The OAuth 2.0 scopes the registration is granted. No OAuth
    /// mechanism exposes the scopes tied to a client registration
    /// (RFC 8414 only lists the server-wide scopes-supported), so
    /// they are hardcoded here, exactly as Thunderbird hardcodes its
    /// own. They fill the config when discovery yielded none.
    scopes: &'static [&'static str],
}

/// The well-known public applications. Thunderbird covers Google,
/// Microsoft and Fastmail today; Pimalaya applications join the list
/// as their provider registrations land.
const KNOWN_APPS: &[KnownApp] = &[
    KnownApp {
        name: "Thunderbird",
        covers: "emails, contacts, calendars",
        host: "accounts.google.com",
        client_id: "406964657835-aq8lmia8j95dhl1a2bvharmfk3t1hgqj.apps.googleusercontent.com",
        client_secret: Some("kSmqreRr0qwBWJgbf5Y-PjSU"),
        redirection: Some("http://localhost"),
        scopes: &[
            "https://mail.google.com/",
            "https://www.googleapis.com/auth/carddav",
            "https://www.googleapis.com/auth/calendar",
        ],
    },
    KnownApp {
        name: "Thunderbird",
        covers: "emails",
        host: "login.microsoftonline.com",
        client_id: "9e5f94bc-e8a4-4e73-b8be-63364c29d753",
        client_secret: None,
        redirection: Some("https://localhost"),
        scopes: &[
            "https://outlook.office.com/IMAP.AccessAsUser.All",
            "https://outlook.office.com/POP.AccessAsUser.All",
            "https://outlook.office.com/SMTP.Send",
            "offline_access",
        ],
    },
    KnownApp {
        name: "Thunderbird",
        covers: "emails, contacts, calendars",
        host: "api.fastmail.com",
        client_id: "35f141ae",
        client_secret: None,
        redirection: None,
        scopes: &[
            "https://www.fastmail.com/dev/protocol-imap",
            "https://www.fastmail.com/dev/protocol-pop",
            "https://www.fastmail.com/dev/protocol-smtp",
            "https://www.fastmail.com/dev/protocol-carddav",
            "https://www.fastmail.com/dev/protocol-caldav",
        ],
    },
];

/// The well-known public applications registered against the same
/// authorization server as the config's endpoints.
fn known_apps(config: &OauthConfig) -> Vec<&'static KnownApp> {
    let endpoints = &config.endpoints;
    let urls = [
        &endpoints.authorization,
        &endpoints.device_authorization,
        &endpoints.token,
    ];

    let hosts: BTreeSet<String> = urls
        .into_iter()
        .flatten()
        .filter_map(|url| Url::parse(url).ok())
        .filter_map(|url| url.host_str().map(str::to_ascii_lowercase))
        .collect();

    KNOWN_APPS
        .iter()
        .filter(|app| hosts.contains(app.host))
        .collect()
}

/// One entry in the token storage pick list: a well-known credential
/// provider CLI, or the trailing custom entry.
#[derive(Debug, Eq, PartialEq)]
enum StorageChoice {
    Known(KnownStorage),
    Custom,
}

impl fmt::Display for StorageChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(provider) => write!(f, "{}", provider.name()),
            Self::Custom => write!(f, "Custom commands"),
        }
    }
}

/// A well-known credential provider CLI the wizard can plug the
/// token storage into.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KnownStorage {
    /// pass, the standard unix password manager.
    Pass,
    /// secret-tool, over the Secret Service (GNOME Keyring).
    SecretTool,
    /// kwallet-query, over the KDE Wallet.
    KwalletQuery,
    /// security, over the macOS Keychain.
    Security,
}

impl KnownStorage {
    /// The providers relevant on the running OS. Empty on platforms
    /// without a known stdin-friendly provider (Windows), where the
    /// wizard goes straight to custom commands.
    fn available() -> Vec<Self> {
        let mut providers = Vec::new();

        if cfg!(target_os = "linux") {
            providers.push(Self::SecretTool);
            providers.push(Self::KwalletQuery);
        }

        if cfg!(target_os = "macos") {
            providers.push(Self::Security);
        }

        if cfg!(unix) {
            providers.push(Self::Pass);
        }

        providers
    }

    /// Display name of the provider, for the pick-list labels.
    fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass (password store)",
            Self::SecretTool => "secret-tool (GNOME Keyring / Secret Service)",
            Self::KwalletQuery => "kwallet-query (KDE Wallet)",
            Self::Security => "security (macOS Keychain)",
        }
    }

    /// The command printing the persisted token on stdout.
    fn read(self, account: &str) -> StorageCommand {
        let exec =
            |args: &[&str]| StorageCommand::Exec(args.iter().map(|s| s.to_string()).collect());

        match self {
            Self::Pass => exec(&["pass", "show", &format!("ortie/{account}")]),
            Self::SecretTool => exec(&[
                "secret-tool",
                "lookup",
                "service",
                "ortie",
                "account",
                account,
            ]),
            Self::KwalletQuery => exec(&[
                "kwallet-query",
                "-r",
                &format!("ortie/{account}"),
                "kdewallet",
            ]),
            Self::Security => exec(&[
                "security",
                "find-generic-password",
                "-s",
                "ortie",
                "-a",
                account,
                "-w",
            ]),
        }
    }

    /// The command persisting the token it receives on stdin.
    fn write(self, account: &str) -> StorageCommand {
        let exec =
            |args: &[&str]| StorageCommand::Exec(args.iter().map(|s| s.to_string()).collect());

        match self {
            Self::Pass => StorageCommand::Shell(format!("pass insert -m -f ortie/{account}")),
            Self::SecretTool => exec(&[
                "secret-tool",
                "store",
                "--label",
                &format!("ortie/{account}"),
                "service",
                "ortie",
                "account",
                account,
            ]),
            Self::KwalletQuery => exec(&[
                "kwallet-query",
                "-w",
                &format!("ortie/{account}"),
                "kdewallet",
            ]),
            // NOTE: security takes the secret as an argument, not on
            // stdin; the shell form bridges it with $(cat).
            Self::Security => StorageCommand::Shell(format!(
                "security add-generic-password -U -s ortie -a {account} -w \"$(cat)\""
            )),
        }
    }
}

/// The account resolved by the wizard, printed as a complete config
/// fragment: valid TOML on stdout with guidance embedded as comments,
/// or the same data as an object in JSON mode.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct OauthConfig {
    /// The account name, heading the `[accounts.<name>]` table.
    name: String,
    /// The OAuth 2.0 client identifier, when already registered.
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    /// The client secret paired with the identifier, for providers
    /// issuing one.
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<RawSecret>,
    /// The wire name of the discovered grant flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    grant: Option<&'static str>,
    /// The discovered endpoints.
    endpoints: Endpoints,
    /// The discovered scopes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    scopes: Vec<String>,
    /// Whether token show refreshes an expired token by itself; the
    /// wizard always enables it.
    auto_refresh: bool,
    /// The bare issuer, when discovery could not resolve endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,
    /// The commands persisting and reading back the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    storage: Option<Storage>,
}

/// The client secret in the config's secret shape
/// (client-secret.raw).
#[derive(Debug, Serialize)]
struct RawSecret {
    raw: String,
}

/// Storage subset of the account config fragment.
#[derive(Debug, Serialize)]
struct Storage {
    read: StorageEntry,
    write: StorageEntry,
}

/// One direction of the token storage, holding its command.
#[derive(Debug, Serialize)]
struct StorageEntry {
    command: StorageCommand,
}

/// One storage command in either accepted config shape: exec-style
/// array, or string wrapped through the platform shell.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
enum StorageCommand {
    Exec(Vec<String>),
    Shell(String),
}

impl fmt::Display for StorageCommand {
    /// Renders the command as its TOML value: a double-quoted string
    /// array for the exec form, a literal (single-quoted) string for
    /// the shell form so embedded quotes need no escaping.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exec(args) => {
                let args: Vec<String> = args.iter().map(|arg| format!("\"{arg}\"")).collect();
                write!(f, "[{}]", args.join(", "))
            }
            Self::Shell(command) => write!(f, "'{command}'"),
        }
    }
}

/// Endpoint subset of the account config fragment.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Endpoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirection: Option<String>,
}

impl fmt::Display for OauthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# OAuth 2.0 account discovered by the ortie wizard.")?;
        writeln!(f, "#")?;
        writeln!(f, "# Save this fragment into your config file, one of:")?;
        writeln!(f, "#   $XDG_CONFIG_HOME/ortie/config.toml")?;
        writeln!(f, "#   $HOME/.config/ortie/config.toml")?;
        writeln!(f, "#   $HOME/.ortierc")?;
        writeln!(
            f,
            "# Prompts render on stderr, so appending works directly:"
        )?;
        writeln!(f, "#   ortie >> ~/.config/ortie/config.toml")?;
        writeln!(f, "#")?;
        writeln!(f, "# Every field is documented in the sample config:")?;
        writeln!(
            f,
            "# https://github.com/pimalaya/ortie/blob/master/config.sample.toml"
        )?;
        writeln!(f, "#")?;
        writeln!(f, "# Complete the commented fields, then:")?;
        writeln!(
            f,
            "#   ortie auth get          issue the first access token"
        )?;
        writeln!(
            f,
            "#   ortie auth resume <url> finish the flow by hand when the redirection fails"
        )?;
        writeln!(
            f,
            "#   ortie token show        print the stored access token"
        )?;
        writeln!(f, "#   ortie token refresh     force a refresh")?;
        writeln!(f)?;
        writeln!(f, "[accounts.{}]", toml_key(&self.name))?;

        match &self.client_id {
            Some(id) => writeln!(f, "client-id = \"{id}\"")?,
            None => {
                writeln!(f, "# Register an OAuth 2.0 client with your provider, or")?;
                writeln!(f, "# reuse a public one (see the README).")?;
                writeln!(f, "client-id = \"\"")?;
            }
        }

        if let Some(secret) = &self.client_secret {
            writeln!(f, "client-secret.raw = \"{}\"", secret.raw)?;
        }

        if let Some(grant) = &self.grant {
            writeln!(f, "grant = \"{grant}\"")?;
        }
        if let Some(url) = &self.endpoints.authorization {
            writeln!(f, "endpoints.authorization = \"{url}\"")?;
        }
        if let Some(url) = &self.endpoints.device_authorization {
            writeln!(f, "endpoints.device-authorization = \"{url}\"")?;
        }
        if let Some(url) = &self.endpoints.token {
            writeln!(f, "endpoints.token = \"{url}\"")?;
        }
        if let Some(url) = &self.endpoints.redirection {
            writeln!(f, "endpoints.redirection = \"{url}\"")?;
        }

        if !self.scopes.is_empty() {
            let scopes: Vec<String> = self
                .scopes
                .iter()
                .map(|scope| format!("\"{scope}\""))
                .collect();
            writeln!(f, "scopes = [{}]", scopes.join(", "))?;
        }

        if let Some(issuer) = &self.issuer {
            writeln!(f, "# Discovery stopped at the OAuth 2.0 issuer below; fill")?;
            writeln!(f, "# the endpoints by hand.")?;
            writeln!(f, "# issuer: {issuer}")?;
        }

        writeln!(f, "auto-refresh = {}", self.auto_refresh)?;

        match &self.storage {
            Some(storage) => {
                writeln!(f, "storage.read.command = {}", storage.read.command)?;
                writeln!(f, "storage.write.command = {}", storage.write.command)
            }
            None => {
                writeln!(f, "# Fill with your secret manager commands; both the")?;
                writeln!(f, "# shell string and the exec array forms work (see")?;
                writeln!(f, "# the sample config).")?;
                writeln!(f, "storage.read.command = \"\"")?;
                writeln!(f, "storage.write.command = \"\"")
            }
        }
    }
}

/// Quotes an account name into a valid TOML table key when it is not
/// a bare key (letters, digits, dashes and underscores only).
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
fn split_scopes(scope: Option<String>) -> Vec<String> {
    scope
        .map(|scope| scope.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default()
}

/// Lowercase wire name of a service, for the pick-list labels.
fn service_name(service: Service) -> &'static str {
    match service {
        Service::Imap => "imap",
        Service::Pop3 => "pop3",
        Service::Smtp => "smtp",
        Service::Jmap => "jmap",
        Service::Caldav => "caldav",
        Service::Carddav => "carddav",
        Service::Webdav => "webdav",
        Service::Managesieve => "managesieve",
    }
}
