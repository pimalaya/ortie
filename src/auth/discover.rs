//! `auth discover` subcommand: interactive OAuth 2.0 account wizard,
//! also run by bare `ortie`.
//!
//! Prompts for an email address, a server or an issuer URI, discovers
//! the PIM services reachable for it through io-pim-discovery,
//! reduces them to the OAuth 2.0 grants they advertise (each tagged
//! with the services sharing it), and lets the user pick one. A
//! trailing manual entry falls back to typing the OAuth 2.0 endpoints
//! by hand. The application step offers every way to obtain a client
//! registration, in io-oauth's preference order: dynamic registration
//! (RFC 7591) first when the provider advertises it, then well-known
//! public applications registered against the discovered provider
//! (Thunderbird today), then a custom entry prompting for everything
//! a manually registered application needs (client id and secret,
//! scopes, redirection endpoint). The storage step plugs the
//! token into a credential provider CLI known for the running OS,
//! falling back to custom shell commands. The outcome is a complete
//! account config fragment printed as valid TOML on stdout (guidance
//! embedded as comments, prompts on stderr), so `ortie >> <config>`
//! appends it.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use anyhow::{Result, bail};
use clap::Parser;
use log::debug;
use pimalaya_cli::{printer::Printer, prompt, spinner::Spinner};
use pimalaya_stream::tls::{Rustls, Tls};
use secrecy::ExposeSecret;
use serde::Serialize;
use url::Url;

use io_oauth::{
    client::Oauth20ClientStd,
    rfc7591::{
        register::{
            Oauth20ClientRegisterErrorCode, Oauth20ClientRegisterParams,
            Oauth20ClientRegisterResponse,
        },
        source::Oauth20ClientSource,
    },
};
use io_pim_discovery::{
    compose::{
        client::DiscoveryComposeClientStd,
        config::{DiscoveryAuthMethod, DiscoveryService, DiscoveryServiceConfig},
    },
    shared::dns::system_resolver,
};

/// Fallback DNS resolver when the system one cannot be determined.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Loopback redirection URI registered by default: RFC 8252
/// section 7.3 lets the port vary at authorization time, so it
/// matches the runtime ephemeral-port default.
const REDIRECT_LOOPBACK: &str = "http://127.0.0.1";

/// Reverse-DNS private-use redirection URI (RFC 8252 section 7.1),
/// retried when the provider rejects http redirections altogether
/// (Fastmail's dynamic registration accepts only private-use
/// schemes). The browser dead-ends on it, so `auth get` prints the
/// manual `auth resume` steps rather than binding a listener.
const REDIRECT_SCHEME: &str = "org.pimalaya.ortie://redirect";

/// Discover OAuth 2.0 services and print a ready-to-append account
/// config fragment. This is also what bare `ortie` runs.
///
/// The wizard prompts for an email address, a server or an issuer
/// URI, discovers the reachable services, reduces them to their
/// OAuth 2.0 grants, lets you pick one (or enter the endpoints
/// manually), then prints the resulting account as valid TOML on
/// stdout. It never writes any file: append the fragment to your
/// config yourself, e.g. `ortie >> <config>`.
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

        // Account name, prompted right after the input: suggest the
        // first label of the input's domain (or URI host).
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
        let name = name.to_string();

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

        // Fill the defaults a provider is known to need but discovery
        // does not yet surface (Fastmail's RFC 8707 resource and its
        // scopes). Stopgap; see docs/discovery-layering.md.
        fill_provider_defaults(&mut config);
        config.name = name;

        // Let the user choose scopes: offer the discovered ones plus
        // any extra the provider is known to advertise, with the
        // discovered set selected by default. Skipped when there is
        // nothing to choose from. The advertised extras are a stopgap
        // until discovery carries scopes_supported; see
        // docs/discovery-layering.md.
        let mut options = config.scopes.clone();
        for scope in advertised_scopes(&config.endpoints) {
            if !options.iter().any(|option| option.as_str() == scope) {
                options.push(scope.to_string());
            }
        }

        if !options.is_empty() {
            let selected: Vec<usize> = options
                .iter()
                .enumerate()
                .filter_map(|(index, option)| config.scopes.contains(option).then_some(index))
                .collect();

            config.scopes = prompt::items("Scopes:", options, selected)?;
        }

        // NOTE: the application step offers every way to obtain a
        // client registration, sorted by io-oauth's client source
        // preference: dynamic registration when the provider
        // advertises it, well-known public applications registered
        // against the same authorization server, then the custom
        // entry.
        let mut choices = Vec::new();
        choices.extend(registration_endpoint(&config).map(ClientChoice::Dynamic));
        choices.extend(known_apps(&config).into_iter().map(ClientChoice::Known));
        choices.push(ClientChoice::Custom);
        choices.sort_by_key(ClientChoice::source);

        if choices.len() == 1 {
            custom_client(&mut config)?;
        } else {
            loop {
                match prompt::item("Application:", choices.clone(), None)? {
                    ClientChoice::Dynamic(endpoint) => match register(&mut config, &endpoint) {
                        Ok(()) => break,
                        // NOTE: the failure was reported by the
                        // register spinner; drop the entry and offer
                        // the remaining ones.
                        Err(_) => {
                            choices.retain(|choice| !matches!(choice, ClientChoice::Dynamic(_)));
                        }
                    },
                    ClientChoice::Known(app) => {
                        config.client_id = Some(app.client_id.to_string());
                        config.client_secret = app.client_secret.map(|raw| RawSecret {
                            raw: raw.to_string(),
                        });
                        config.endpoints.redirection = app.redirection.map(ToString::to_string);

                        // NOTE: discovered scopes stay, they are
                        // narrower (per service); the app's registered
                        // set only fills the gap when discovery
                        // yielded none.
                        if config.scopes.is_empty() {
                            config.scopes = app.scopes.iter().map(ToString::to_string).collect();
                        }

                        break;
                    }
                    ClientChoice::Custom => {
                        custom_client(&mut config)?;
                        break;
                    }
                }
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
/// the OAuth 2.0 grants found (or the trailing manual entry). Falls
/// straight through to manual entry when nothing is found.
fn choose(email: &str) -> Result<OauthConfig> {
    let spinner = Spinner::start("Searching for OAuth 2.0 grants");
    let discovered = discover(email)?;

    if discovered.is_empty() {
        spinner.failure("No OAuth 2.0 grant found, entering manually");
        return manual(None);
    }

    spinner.success(format!("Found {} OAuth 2.0 grant(s)", discovered.len()));

    let mut choices: Vec<Choice> = discovered.into_iter().map(Choice::Discovered).collect();
    choices.push(Choice::Manual);

    match prompt::item("Choose an OAuth 2.0 grant:", choices, None)? {
        Choice::Discovered(grant) => Ok(grant.into_config()),
        Choice::Manual => manual(None),
    }
}

/// Composes service configs for `email` across every discovery
/// mechanism and reduces the result to the deduplicated OAuth 2.0
/// methods it advertises, each tagged with the services that share it.
fn discover(email: &str) -> Result<Vec<DiscoveredOauth>> {
    let client = compose_client();

    // The OAuth-capable PIM services; POP3, WebDAV and ManageSieve
    // never advertise an OAuth flow of their own.
    let services = BTreeSet::from([
        DiscoveryService::Imap,
        DiscoveryService::Smtp,
        DiscoveryService::Jmap,
        DiscoveryService::Caldav,
        DiscoveryService::Carddav,
    ]);

    debug!("compose OAuth 2.0 services for {email}");
    let configs = client.compose_all(email, services)?;

    Ok(collect_oauth(&configs))
}

/// The discovery client shared by the wizard's network steps, backed
/// by the system DNS resolver (with a public fallback).
fn compose_client() -> DiscoveryComposeClientStd {
    let resolver = system_resolver().unwrap_or_else(|| {
        DEFAULT_RESOLVER
            .parse()
            .expect("default resolver must be a valid URL")
    });

    DiscoveryComposeClientStd::new(resolver, wizard_tls())
}

/// TLS options for the wizard's HTTPS calls, pinned to HTTP/1.1.
fn wizard_tls() -> Tls {
    Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".to_string()],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Collects the OAuth 2.0 methods across every discovered config,
/// grouped by flow and endpoints, each carrying the union of the
/// scopes and the set of services it authenticates. Per-service grants
/// that differ only in scope (Microsoft's IMAP and SMTP, say) merge
/// into one entry, so a single token can cover every service.
fn collect_oauth(configs: &[DiscoveryServiceConfig]) -> Vec<DiscoveredOauth> {
    let mut discovered: Vec<DiscoveredOauth> = Vec::new();

    for config in configs {
        for method in &config.auth {
            if !is_oauth(method) {
                continue;
            }

            match discovered
                .iter_mut()
                .find(|d| same_grant(&d.method, method))
            {
                Some(existing) => {
                    merge_scopes(&mut existing.method, method);
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

/// Whether two grants are the same flow against the same endpoints,
/// ignoring their scope, so per-service grants merge into one.
fn same_grant(a: &DiscoveryAuthMethod, b: &DiscoveryAuthMethod) -> bool {
    match (a, b) {
        (
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: a_authorization,
                token_endpoint: a_token,
                ..
            },
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: b_authorization,
                token_endpoint: b_token,
                ..
            },
        ) => a_authorization == b_authorization && a_token == b_token,
        (
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: a_device,
                token_endpoint: a_token,
                ..
            },
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: b_device,
                token_endpoint: b_token,
                ..
            },
        ) => a_device == b_device && a_token == b_token,
        (DiscoveryAuthMethod::OauthIssuer(a), DiscoveryAuthMethod::OauthIssuer(b)) => a == b,
        _ => false,
    }
}

/// Unions the incoming grant's scope tokens into the existing grant's,
/// preserving order and dropping duplicates, so a merged grant
/// requests every grouped service's scopes at once.
fn merge_scopes(existing: &mut DiscoveryAuthMethod, incoming: &DiscoveryAuthMethod) {
    let existing_scope = match existing {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { scope, .. }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { scope, .. } => scope,
        _ => return,
    };

    let incoming_scope = match incoming {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { scope, .. }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { scope, .. } => scope,
        _ => return,
    };

    let mut tokens: Vec<String> = existing_scope
        .as_deref()
        .map(|scope| scope.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default();

    if let Some(incoming) = incoming_scope.as_deref() {
        for token in incoming.split_whitespace() {
            if !tokens.iter().any(|existing| existing == token) {
                tokens.push(token.to_string());
            }
        }
    }

    *existing_scope = (!tokens.is_empty()).then(|| tokens.join(" "));
}

/// Whether an authentication method is one of the OAuth 2.0 flows.
fn is_oauth(method: &DiscoveryAuthMethod) -> bool {
    matches!(
        method,
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. }
            | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { .. }
            | DiscoveryAuthMethod::OauthIssuer(_)
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
        extras: BTreeMap::new(),
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

/// The provider's RFC 7591 registration endpoint, when it advertises
/// one in its RFC 8414 metadata.
///
/// No discovery mechanism carries this information (the compose
/// layer keeps flow endpoints only, and the autoconfig sources never
/// see server metadata), so the wizard asks the provider itself: the
/// issuer is guessed from each endpoint host (https://<host>) and
/// the first metadata advertising a registration endpoint wins.
fn registration_endpoint(config: &OauthConfig) -> Option<Url> {
    let spinner = Spinner::start("Checking for dynamic client registration");
    let client = compose_client();

    for host in endpoint_hosts(&config.endpoints) {
        let Ok(issuer) = format!("https://{host}").parse::<Url>() else {
            continue;
        };

        let Some(metadata) = client.oauth_server(&issuer) else {
            continue;
        };

        if let Some(endpoint) = metadata.registration_endpoint {
            spinner.success("Dynamic client registration advertised");
            return Some(endpoint);
        }
    }

    spinner.failure("No dynamic client registration advertised");
    None
}

/// Registers ortie dynamically against the provider's registration
/// endpoint (RFC 7591): a public client without secret
/// (token_endpoint_auth_method none), the grant types of the
/// discovered flow and the discovered scopes. The issued client id
/// (and secret, when the server insists on one) land in the config
/// fragment.
///
/// A loopback redirection is registered first, matching the runtime
/// default; providers rejecting http redirections altogether
/// (Fastmail's dynamic registration accepts only private-use schemes)
/// get a reverse-DNS private-use scheme instead, which the fragment
/// then pins so `auth get` hands off to a manual `auth resume`.
fn register(config: &mut OauthConfig, endpoint: &Url) -> Result<()> {
    let device = config.grant == Some("device");
    let scopes = config.scopes.join(" ");

    let mut params = Oauth20ClientRegisterParams {
        redirect_uris: if device {
            Vec::new()
        } else {
            vec![REDIRECT_LOOPBACK.to_string()]
        },
        token_endpoint_auth_method: Some("none".to_string()),
        grant_types: if device {
            vec![
                "urn:ietf:params:oauth:grant-type:device_code".to_string(),
                "refresh_token".to_string(),
            ]
        } else {
            vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ]
        },
        response_types: if device {
            Vec::new()
        } else {
            vec!["code".to_string()]
        },
        client_name: Some("Ortie".to_string()),
        scope: (!scopes.is_empty()).then_some(scopes),
        ..Default::default()
    };

    let tls = wizard_tls();
    let spinner = Spinner::start("Registering ortie as a public client");

    let mut response = register_once(endpoint, &tls, &params);

    // NOTE: some providers (Fastmail) reject every http redirection,
    // loopback included, and only accept a reverse-DNS private-use
    // scheme (RFC 8252 section 7.1); retry with one before giving up.
    if let Ok(Err(rejection)) = &response {
        let redirect_rejected =
            rejection.error == Oauth20ClientRegisterErrorCode::InvalidRedirectUri;

        if redirect_rejected && !device {
            params.redirect_uris = vec![REDIRECT_SCHEME.to_string()];
            response = register_once(endpoint, &tls, &params);
        }
    }

    match response {
        Ok(Ok(client)) => {
            spinner.success(format!("Registered client {}", client.client_id));

            config.client_id = Some(client.client_id);
            config.client_secret = client.client_secret.map(|secret| RawSecret {
                raw: secret.expose_secret().to_string(),
            });

            // NOTE: the loopback registration matches the runtime
            // default (ephemeral 127.0.0.1 port, free per RFC 8252
            // section 7.3), so only the private-use scheme needs
            // pinning.
            if params.redirect_uris.first().map(String::as_str) == Some(REDIRECT_SCHEME) {
                config.endpoints.redirection = Some(REDIRECT_SCHEME.to_string());
            }

            Ok(())
        }
        Ok(Err(rejection)) => {
            let detail = rejection
                .error_description
                .unwrap_or_else(|| format!("{:?}", rejection.error));
            spinner.failure(format!("Registration rejected: {detail}"));
            bail!("Registration rejected: {detail}");
        }
        Err(err) => {
            spinner.failure(format!("Registration failed: {err}"));
            Err(err)
        }
    }
}

/// Posts one registration attempt over a fresh connection to the
/// registration endpoint; servers rarely keep the socket alive, so
/// the redirect-scheme retry reconnects instead of reusing a stream.
fn register_once(
    endpoint: &Url,
    tls: &Tls,
    params: &Oauth20ClientRegisterParams,
) -> Result<Oauth20ClientRegisterResponse> {
    // NOTE: no client id exists yet, registration is what issues it.
    let mut client = Oauth20ClientStd::connect(endpoint.clone(), tls, "")?;
    let response = client.register_client(endpoint, params)?;

    Ok(response)
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
    method: DiscoveryAuthMethod,
    services: BTreeSet<DiscoveryService>,
}

impl DiscoveredOauth {
    fn into_config(self) -> OauthConfig {
        match self.method {
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
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
                extras: BTreeMap::new(),
                auto_refresh: true,
                issuer: None,
                storage: None,
            },
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
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
                extras: BTreeMap::new(),
                auto_refresh: true,
                issuer: None,
                storage: None,
            },
            DiscoveryAuthMethod::OauthIssuer(issuer) => OauthConfig {
                name: String::new(),
                client_id: None,
                client_secret: None,
                grant: None,
                endpoints: Endpoints::default(),
                scopes: Vec::new(),
                extras: BTreeMap::new(),
                auto_refresh: true,
                issuer: Some(issuer),
                storage: None,
            },
            // NOTE: collect_oauth only keeps the OAuth variants above.
            _ => unreachable!("collect_oauth retains OAuth methods only"),
        }
    }
}

/// One entry in the grant pick list: a discovered grant, or the
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
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant { token_endpoint, .. } => {
                write!(f, "OAuth 2.0 authorization code grant")?;
                write!(f, " ({services}) via {token_endpoint}")
            }
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { token_endpoint, .. } => {
                write!(f, "OAuth 2.0 device authorization grant")?;
                write!(f, " ({services}) via {token_endpoint}")
            }
            DiscoveryAuthMethod::OauthIssuer(issuer) => {
                write!(f, "OAuth 2.0 issuer {issuer} ({services})")
            }
            _ => Ok(()),
        }
    }
}

/// One entry in the application pick list: dynamic registration
/// against the provider's advertised endpoint, a well-known public
/// application, or the trailing custom entry.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ClientChoice {
    Dynamic(Url),
    Known(&'static KnownApp),
    Custom,
}

impl ClientChoice {
    /// The io-oauth client source of the entry, whose derived order
    /// (dynamic registration, public client, manual) is the
    /// pick-list preference.
    fn source(&self) -> Oauth20ClientSource {
        match self {
            Self::Dynamic(_) => Oauth20ClientSource::DynamicRegistration,
            Self::Known(_) => Oauth20ClientSource::PublicClient,
            Self::Custom => Oauth20ClientSource::Manual,
        }
    }
}

impl fmt::Display for ClientChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dynamic(endpoint) => write!(f, "Dynamic registration via {endpoint}"),
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
    let hosts = endpoint_hosts(&config.endpoints);

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
    /// Extra authorization-request parameters a provider is known to
    /// require but discovery does not yet surface (Fastmail's RFC 8707
    /// resource). Stopgap; see docs/discovery-layering.md.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    extras: BTreeMap<String, String>,
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

        if !self.extras.is_empty() {
            writeln!(
                f,
                "# Extra authorization parameters this provider requires."
            )?;
            for (key, value) in &self.extras {
                writeln!(f, "extras.{key} = \"{value}\"")?;
            }
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

/// Fills the defaults a provider is known to need but discovery does
/// not yet surface. Fastmail's authorization endpoint bounces the flow
/// pre-consent (no password or scope screen, a straight redirect to
/// the "close this window" page) unless the RFC 8707 resource
/// indicator is present, and its discovered grant carries no scopes at
/// all; supply the resource and, since Fastmail cannot complete on a
/// desktop anyway, its full advertised scope set. Stopgap until
/// discovery surfaces them; see docs/discovery-layering.md.
fn fill_provider_defaults(config: &mut OauthConfig) {
    let hosts = endpoint_hosts(&config.endpoints);

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

/// The scopes a provider is known to advertise, offered to the user as
/// a multi-select. Empty for providers whose scopes discovery already
/// fills (their config keeps the discovered set). Stopgap until
/// discovery carries scopes_supported; see docs/discovery-layering.md.
fn advertised_scopes(endpoints: &Endpoints) -> Vec<&'static str> {
    let hosts = endpoint_hosts(endpoints);

    if hosts.contains("api.fastmail.com") {
        return vec![
            "urn:ietf:params:oauth:scope:mail",
            "urn:ietf:params:oauth:scope:contacts",
            "urn:ietf:params:oauth:scope:calendars",
            "offline_access",
        ];
    }

    Vec::new()
}

/// The distinct hosts of the config's endpoints, lowercased.
fn endpoint_hosts(endpoints: &Endpoints) -> BTreeSet<String> {
    let urls = [
        &endpoints.authorization,
        &endpoints.device_authorization,
        &endpoints.token,
    ];

    urls.into_iter()
        .flatten()
        .filter_map(|url| Url::parse(url).ok())
        .filter_map(|url| url.host_str().map(str::to_ascii_lowercase))
        .collect()
}

/// Splits a space-separated scope string into the config list shape.
fn split_scopes(scope: Option<String>) -> Vec<String> {
    scope
        .map(|scope| scope.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default()
}

/// Lowercase wire name of a service, for the pick-list labels.
fn service_name(service: DiscoveryService) -> &'static str {
    match service {
        DiscoveryService::Imap => "imap",
        DiscoveryService::Pop3 => "pop3",
        DiscoveryService::Smtp => "smtp",
        DiscoveryService::Jmap => "jmap",
        DiscoveryService::Caldav => "caldav",
        DiscoveryService::Carddav => "carddav",
        DiscoveryService::Webdav => "webdav",
        DiscoveryService::Managesieve => "managesieve",
    }
}
