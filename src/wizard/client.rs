//! Application step of the wizard: how the account obtains an OAuth
//! 2.0 client registration.
//!
//! Every way to obtain one is offered at once, sorted by io-oauth's
//! client source preference: dynamic registration (RFC 7591) when the
//! authorization server advertises a registration endpoint, then the
//! well-known public applications registered against that same server,
//! then a custom entry for a client the user registered by hand.
//!
//! The custom entry prompts for nothing. A registration of one's own
//! is the rare case, and the person who made one is already editing
//! the config: typing a client id, a secret and a redirection into a
//! wizard only to check them in a file afterwards helps nobody. It
//! yields the account with its client id left empty, and the wizard
//! explains what to fill in.
//!
//! The step runs before the scope step and hands it the options its
//! outcome allows (see [`scope`]), since a registration is what
//! decides which scopes can be requested at all.

use std::fmt;

use anyhow::{Result, bail};
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
use io_pim_discovery::rfc8414::DiscoveryOauthServerMetadata;
use pimalaya_cli::{prompt, spinner::Spinner};
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use url::Url;

use crate::wizard::{OauthConfig, RawSecret, scope, search};

/// Loopback redirection URI registered by default: RFC 8252
/// section 7.3 lets the port vary at authorization time, so it matches
/// the runtime ephemeral-port default.
const REDIRECT_LOOPBACK: &str = "http://127.0.0.1";

/// Reverse-DNS private-use redirection URI (RFC 8252 section 7.1),
/// retried when the provider rejects http redirections altogether
/// (Fastmail's dynamic registration accepts only private-use schemes).
/// The browser dead-ends on it, so `auth get` prints the manual
/// `auth resume` steps rather than binding a listener.
const REDIRECT_SCHEME: &str = "org.pimalaya.ortie://redirect";

/// Runs the application step against `config`, filling in its client
/// id, client secret and (when the provider pins one) redirection
/// endpoint, and returns the scope options the chosen application
/// allows.
///
/// `metadata` is the run's authorization server metadata, if any: its
/// registration endpoint is what decides whether dynamic registration
/// is on offer, and its advertised scopes are what a client registered
/// for this account may ask for. A single candidate skips the pick
/// list.
pub fn configure(
    config: &mut OauthConfig,
    metadata: Option<&DiscoveryOauthServerMetadata>,
) -> Result<scope::Source> {
    let registration_endpoint =
        metadata.and_then(|metadata| metadata.registration_endpoint.clone());

    let mut choices = Vec::new();
    choices.extend(registration_endpoint.map(Choice::Dynamic));
    choices.extend(known_apps(config).into_iter().map(Choice::Known));
    choices.push(Choice::Custom);
    choices.sort_by_key(Choice::source);

    // NOTE: the custom entry alone is no choice, and it asks nothing,
    // so the whole step is skipped rather than shown.
    if choices.len() == 1 {
        return Ok(scope::Source::Taken);
    }

    loop {
        match prompt::item("Application:", choices.clone(), None)? {
            Choice::Dynamic(endpoint) => {
                // NOTE: the scopes travel inside the registration
                // request, so they are picked before it is sent.
                scope::prompt(config, scope::advertised(config, metadata))?;

                match register(config, &endpoint) {
                    Ok(()) => return Ok(scope::Source::Taken),
                    // NOTE: the failure was reported by the register
                    // spinner; drop the entry and offer the rest.
                    Err(_) => choices.retain(|choice| !matches!(choice, Choice::Dynamic(_))),
                }
            }
            Choice::Known(app) => {
                config.client_id = Some(app.client_id.to_string());
                config.client_secret = app.client_secret.map(|raw| RawSecret {
                    raw: raw.to_string(),
                });
                config.endpoints.redirection = app.redirection.map(ToString::to_string);

                // NOTE: the discovered scopes are the narrow per-service
                // ones, so they drive the selection; without them every
                // registered scope is selected instead.
                if config.scopes.is_empty() {
                    config.scopes = app.scopes.iter().map(ToString::to_string).collect();
                }

                let registered = app.scopes.iter().map(ToString::to_string).collect();

                return Ok(scope::Source::Registered(registered));
            }
            // NOTE: nothing to ask. The client id, its secret and its
            // redirection are the user's to fill in, and the scopes
            // stay the discovered ones, since nothing exposes what a
            // hand-made registration was granted.
            Choice::Custom => return Ok(scope::Source::Taken),
        }
    }
}

/// Registers Ortie dynamically against the provider's registration
/// endpoint (RFC 7591): a public client without secret
/// (`token_endpoint_auth_method` none), the grant and response types of
/// the discovered flow, and the discovered scopes. The issued client id
/// (and secret, when the server insists on one) land in the config.
///
/// A loopback redirection is registered first, matching the runtime
/// default; providers rejecting http redirections altogether get a
/// reverse-DNS private-use scheme instead, which the config then pins.
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

    let tls = search::wizard_tls();
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
/// registration endpoint; servers rarely keep the socket alive, so the
/// redirect-scheme retry reconnects instead of reusing a stream.
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

/// One entry in the application pick list: dynamic registration
/// against the provider's advertised endpoint, a well-known public
/// application, or the trailing custom entry.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Choice {
    Dynamic(Url),
    Known(&'static KnownApp),
    Custom,
}

impl Choice {
    /// The io-oauth client source of the entry, whose derived order
    /// (dynamic registration, public client, manual) is the pick-list
    /// preference.
    fn source(&self) -> Oauth20ClientSource {
        match self {
            Self::Dynamic(_) => Oauth20ClientSource::DynamicRegistration,
            Self::Known(_) => Oauth20ClientSource::PublicClient,
            Self::Custom => Oauth20ClientSource::Manual,
        }
    }
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dynamic(endpoint) => write!(f, "Dynamic registration via {endpoint}"),
            Self::Known(app) => write!(f, "{} ({})", app.name, app.covers),
            Self::Custom => write!(f, "Custom application (filled in by hand)"),
        }
    }
}

/// A well-known public application whose client id is reusable.
///
/// Providers bind a registration to their own authorization server, so
/// each entry carries the endpoint host it was registered against and
/// only shows up when the account's endpoints live on that host.
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
    /// mechanism exposes the scopes tied to a client registration (RFC
    /// 8414 only lists the server-wide scopes-supported), so they are
    /// hardcoded here, exactly as Thunderbird hardcodes its own. They
    /// fill the config when discovery yielded none.
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
    let hosts = config.endpoints.hosts();

    KNOWN_APPS
        .iter()
        .filter(|app| hosts.contains(app.host))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::wizard::Endpoints;

    use super::*;

    fn config_on(host: &str) -> OauthConfig {
        OauthConfig {
            endpoints: Endpoints {
                token: Some(format!("https://{host}/token")),
                ..Default::default()
            },
            ..OauthConfig::empty()
        }
    }

    #[test]
    fn known_apps_are_scoped_to_their_authorization_server() {
        assert_eq!(known_apps(&config_on("api.fastmail.com")).len(), 1);
        assert_eq!(known_apps(&config_on("accounts.google.com")).len(), 1);
        assert!(known_apps(&config_on("mail.example.test")).is_empty());
    }

    #[test]
    fn pick_list_follows_the_io_oauth_source_order() {
        let mut choices = [
            Choice::Custom,
            Choice::Known(&KNOWN_APPS[0]),
            Choice::Dynamic("https://as/register".parse().unwrap()),
        ];
        choices.sort_by_key(Choice::source);

        assert!(matches!(choices[0], Choice::Dynamic(_)));
        assert!(matches!(choices[1], Choice::Known(_)));
        assert!(matches!(choices[2], Choice::Custom));
    }
}
