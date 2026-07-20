//! TOML configuration for the `ortie` CLI.
//!
//! All types here are pure DTOs: they mirror the nested TOML shape
//! (`storage.read.command`, `hooks.on-refresh.error.notify`, ...)
//! and carry no behaviour. The merged, flat runtime view that
//! commands consume lives in [`crate::account::Account`].
//!
//! Loaded from the first valid path among:
//! - `$XDG_CONFIG_HOME/ortie/config.toml`
//! - `$HOME/.config/ortie/config.toml`
//! - `$HOME/.ortierc`
//!
//! Override with `-c, --config <PATH>` or `ORTIE_CONFIG=<PATH>`.

use std::{collections::HashMap, fmt, process::Command};

use pimalaya_config::{command, secret::Secret, toml::TomlConfig};
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
#[cfg(feature = "notify")]
use serde::Serialize;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use url::Url;

/// Root of the TOML configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Accounts indexed by name, one per `[accounts.<name>]` block.
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn take_default_account(&mut self) -> Option<(String, Self::Account)> {
        let name = self
            .accounts
            .iter()
            .find_map(|(name, account)| account.default.then(|| name.clone()))?;
        self.accounts.remove_entry(&name)
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts.remove_entry(name)
    }
}

/// One `[accounts.<name>]` block; nested shape mirrors the TOML.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// Whether this account is picked when no `-a <NAME>` is passed.
    #[serde(default)]
    pub default: bool,

    /// OAuth 2.0 client identifier, as registered with the provider.
    pub client_id: String,
    /// Optional OAuth 2.0 client secret; PKCE-only public clients
    /// skip it.
    pub client_secret: Option<Secret>,

    /// OAuth 2.0 grant flow run by the auth commands.
    #[serde(default)]
    pub grant: GrantConfig,
    /// Endpoints of the OAuth 2.0 authorization server.
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    /// TLS provider used for the HTTPS connections.
    #[serde(default, deserialize_with = "tls")]
    pub tls: Tls,

    /// OAuth 2.0 scopes requested for the access token.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// PKCE posture of the authorization code grant.
    #[serde(default)]
    pub pkce: PkceConfig,
    /// Extra parameters forwarded verbatim to the authorization
    /// request query; keys are wire names, never kebab-renamed.
    #[serde(default)]
    pub extras: HashMap<String, String>,
    /// Whether `token show` refreshes an expired token by itself.
    #[serde(default)]
    pub auto_refresh: bool,

    /// Shell commands reading and writing the persisted token.
    pub storage: StoragesConfig,
    /// Shell commands and notifications fired on issue and refresh.
    #[serde(default)]
    pub hooks: HooksConfig,
}

/// OAuth 2.0 grant flow run by the auth commands.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum GrantConfig {
    /// The authorization code grant (RFC 6749 section 4.1), the
    /// browser-redirect flow.
    #[default]
    AuthorizationCode,
    /// The device authorization grant (RFC 8628), the user-code flow
    /// for input-constrained hosts.
    Device,
}

/// Endpoints of the OAuth 2.0 authorization server.
///
/// All optional at parse time: each command checks the endpoints it
/// actually needs (auth get needs the configured grant's endpoints,
/// token refresh only the token one, token show none at all).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EndpointsConfig {
    /// Authorization endpoint, where the authorization code grant
    /// sends the user's browser.
    pub authorization: Option<Url>,
    /// Device authorization endpoint (RFC 8628), used when
    /// `grant = "device"`.
    pub device_authorization: Option<Url>,
    /// Token endpoint, where grants and refreshes exchange for a
    /// token.
    pub token: Option<Url>,
    /// Redirection endpoint the provider sends the browser back to.
    /// When omitted, a random `http://127.0.0.1:<port>` is bound.
    pub redirection: Option<Url>,
}

/// PKCE posture of the authorization code grant.
///
/// Accepts both TOML shapes: a boolean (true = S256, false = off) and
/// a method string ("s256" or "plain"). Defaults to S256, aligning
/// with OAuth 2.1 which requires PKCE on every authorization code
/// flow. Ignored by the device grant, which has no PKCE.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PkceConfig {
    /// SHA-256 code challenge method, the OAuth 2.1 default.
    #[default]
    S256,
    /// Plain code challenge method, for servers rejecting S256.
    Plain,
    /// PKCE disabled, for servers rejecting PKCE parameters.
    Off,
}

impl<'de> Deserialize<'de> for PkceConfig {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct PkceVisitor;

        impl Visitor<'_> for PkceVisitor {
            type Value = PkceConfig;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a boolean, \"s256\" or \"plain\"")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(if v { PkceConfig::S256 } else { PkceConfig::Off })
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "s256" => Ok(PkceConfig::S256),
                    "plain" => Ok(PkceConfig::Plain),
                    _ => Err(E::invalid_value(de::Unexpected::Str(v), &self)),
                }
            }
        }

        de.deserialize_any(PkceVisitor)
    }
}

/// The `storage` block: how the token is persisted.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct StoragesConfig {
    /// Command printing the stored token JSON on its stdout.
    pub read: StorageConfig,
    /// Command receiving the token JSON on its stdin.
    pub write: StorageConfig,
}

/// One storage direction, wrapping a single shell command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct StorageConfig {
    /// The shell command, as a `sh -c` string or an exec-style array.
    #[serde(alias = "cmd", with = "command")]
    pub command: Command,
}

/// The `hooks` block, split by triggering event.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HooksConfig {
    /// Hooks fired when a new access token is issued.
    #[serde(default)]
    pub on_issue: HookStatusConfig,
    /// Hooks fired when the access token is refreshed.
    #[serde(default)]
    pub on_refresh: HookStatusConfig,
}

/// Hooks of one event, split by outcome.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HookStatusConfig {
    /// Hook fired on success.
    #[serde(default)]
    pub success: HookConfig,
    /// Hook fired on error.
    #[serde(default)]
    pub error: HookConfig,
}

/// One hook: an optional shell command and an optional notification.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HookConfig {
    /// Shell command receiving the outcome as environment variables.
    #[serde(default, alias = "cmd", deserialize_with = "deserialize_opt_command")]
    pub command: Option<Command>,
    /// System notification with shell-expanded summary and body.
    #[cfg(feature = "notify")]
    #[serde(default)]
    pub notify: Option<NotifyConfig>,
}

/// System notification content; `$VAR` references are expanded from
/// the hook environment variables.
#[cfg(feature = "notify")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct NotifyConfig {
    /// Notification title.
    pub summary: String,
    /// Notification body.
    pub body: String,
}

/// Placeholder keeping the hook shape identical when the notify cargo
/// feature is disabled.
#[cfg(not(feature = "notify"))]
pub type NotifyConfig = ();

fn deserialize_opt_command<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Command>, D::Error> {
    command::deserialize(de).map(Some)
}

/// TLS provider selector, converted into the pimalaya-stream config.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TlsConfig {
    NativeTls,
    RustlsAws,
    RustlsRing,
}

impl From<TlsConfig> for Tls {
    fn from(tls: TlsConfig) -> Self {
        match tls {
            TlsConfig::NativeTls => Self {
                provider: Some(TlsProvider::NativeTls),
                rustls: Rustls::default(),
                cert: None,
            },
            TlsConfig::RustlsAws => Self {
                provider: Some(TlsProvider::Rustls),
                rustls: Rustls {
                    crypto: Some(RustlsCrypto::Aws),
                    alpn: Vec::new(),
                },
                cert: None,
            },
            TlsConfig::RustlsRing => Self {
                provider: Some(TlsProvider::Rustls),
                rustls: Rustls {
                    crypto: Some(RustlsCrypto::Ring),
                    alpn: Vec::new(),
                },
                cert: None,
            },
        }
    }
}

fn tls<'de, D: Deserializer<'de>>(d: D) -> Result<Tls, D::Error> {
    Ok(TlsConfig::deserialize(d)?.into())
}
