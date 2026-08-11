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
//! Override with `-c, --config <PATH>`, repeated once per file: the
//! first one is the base and the rest are deep-merged on top.

use std::{collections::HashMap, fmt, path::PathBuf, process::Command};

use pimalaya_config::{command, secret::Secret, toml, toml::TomlConfig};
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
    /// Path to the private key signing JWT client assertions
    /// (PKCS#8 or PKCS#1 PEM), used by `grant =
    /// "client-credentials-jwt"`. Re-read at every mint.
    #[serde(default, deserialize_with = "opt_shell_expanded_path")]
    pub client_key: Option<PathBuf>,
    /// Path to the client certificate (PEM or DER) whose SHA-1
    /// thumbprint rides as the assertion `x5t` header, required by
    /// Microsoft certificate credentials. Recomputed at every mint.
    #[serde(default, deserialize_with = "opt_shell_expanded_path")]
    pub client_certificate: Option<PathBuf>,

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
    /// The client credentials grant (RFC 6749 section 4.4), the
    /// headless machine flow authenticated by the client secret.
    ClientCredentials,
    /// The client credentials grant authenticated by a signed JWT
    /// client assertion (RFC 7523 section 2.2), the Microsoft
    /// certificate credentials flow.
    ClientCredentialsJwt,
}

impl GrantConfig {
    /// Whether this grant is a client credentials kind: fully
    /// headless, no refresh token issued, refreshed by re-running
    /// the grant.
    pub fn is_client_credentials(self) -> bool {
        matches!(self, Self::ClientCredentials | Self::ClientCredentialsJwt)
    }
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

fn opt_shell_expanded_path<'de, D: Deserializer<'de>>(de: D) -> Result<Option<PathBuf>, D::Error> {
    toml::shell_expanded_path(de).map(Some)
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

#[cfg(test)]
mod tests {
    use std::{io::Write, path::PathBuf};

    use super::*;

    fn parse(toml: &str) -> AccountConfig {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(toml.as_bytes()).unwrap();

        let mut config = Config::from_paths(&[file.path().to_path_buf()]).unwrap();
        config.take_named_account("test").unwrap().1
    }

    #[test]
    fn client_credentials_account_parses() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
client-secret.raw = "s3cret"
grant = "client-credentials"
endpoints.token = "https://login.example.com/token"
scopes = ["https://graph.microsoft.com/.default"]
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#,
        );

        assert_eq!(account.grant, GrantConfig::ClientCredentials);
        assert!(account.grant.is_client_credentials());
        assert!(account.client_secret.is_some());
        assert_eq!(account.scopes, ["https://graph.microsoft.com/.default"]);
        assert_eq!(
            account.endpoints.token.unwrap().as_str(),
            "https://login.example.com/token"
        );
    }

    #[test]
    fn client_credentials_jwt_account_parses() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
grant = "client-credentials-jwt"
client-key = "/etc/ortie/key.pem"
client-certificate = "/etc/ortie/cert.pem"
endpoints.token = "https://login.example.com/token"
scopes = ["https://graph.microsoft.com/.default"]
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#,
        );

        assert_eq!(account.grant, GrantConfig::ClientCredentialsJwt);
        assert!(account.grant.is_client_credentials());
        assert!(account.client_secret.is_none());
        assert_eq!(
            account.client_key,
            Some(PathBuf::from("/etc/ortie/key.pem"))
        );
        assert_eq!(
            account.client_certificate,
            Some(PathBuf::from("/etc/ortie/cert.pem"))
        );
    }

    #[test]
    fn interactive_grants_are_not_client_credentials() {
        assert!(!GrantConfig::AuthorizationCode.is_client_credentials());
        assert!(!GrantConfig::Device.is_client_credentials());
    }
}
