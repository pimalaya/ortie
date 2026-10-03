//! # Configuration
//!
//! The TOML configuration of the `ortie` CLI. Every type here is a pure
//! DTO mirroring the nested shape (`storage.read.command`,
//! `hooks.on-refresh.error.notify`, ...), the sole behaviour being
//! [`TlsConfig::into_tls`], which folds the selector and the ALPN list
//! into the handle the connect helpers expect.
//!
//! The flat runtime view commands consume is
//! [`crate::account::Account`], which this one is flattened into once
//! the selected account is taken.
//!
//! Loaded from the first valid path among
//! $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml
//! and $HOME/.ortierc. `-c, --config <PATH>` overrides it, repeated
//! once per file: the first is the base, the rest deep-merge on top.

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
    /// OAuth 2.0 client secret, skipped by PKCE-only public clients.
    pub client_secret: Option<Secret>,
    /// Private key (PKCS#8 or PKCS#1 PEM) signing the JWT assertion of
    /// `grant = "client-credentials-jwt"`, re-read at every mint.
    #[serde(default, deserialize_with = "toml::opt_shell_expanded_path")]
    pub client_key: Option<PathBuf>,
    /// Client certificate (PEM or DER) whose SHA-1 thumbprint rides as
    /// the assertion `x5t` header, recomputed at every mint.
    #[serde(default, deserialize_with = "toml::opt_shell_expanded_path")]
    pub client_certificate: Option<PathBuf>,
    /// OAuth 2.0 grant flow run by the auth commands.
    #[serde(default)]
    pub grant: GrantConfig,
    /// Endpoints of the OAuth 2.0 authorization server.
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    /// TLS provider used for the HTTPS connections.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, empty by
    /// default so no ALPN extension is sent.
    ///
    /// An OAuth 2.0 endpoint is plain HTTPS and registers no
    /// identifier; `["http/1.1"]` is the one meaningful override, for a
    /// middlebox refusing a handshake without ALPN. Only rustls reads it.
    #[serde(default)]
    pub alpn: Vec<String>,
    /// OAuth 2.0 scopes requested for the access token.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// PKCE posture of the authorization code grant.
    #[serde(default)]
    pub pkce: PkceConfig,
    /// Extra parameters forwarded verbatim to the authorization request
    /// query, keyed by wire name and never kebab-renamed.
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
    /// The browser-redirect flow (RFC 6749 section 4.1).
    #[default]
    AuthorizationCode,
    /// The user-code flow for input-constrained hosts (RFC 8628).
    Device,
    /// The headless machine flow authenticated by the client secret
    /// (RFC 6749 section 4.4).
    ClientCredentials,
    /// The headless machine flow authenticated by a signed JWT client
    /// assertion (RFC 7523 section 2.2).
    ClientCredentialsJwt,
}

impl GrantConfig {
    /// Whether this grant is a client credentials kind: headless, and
    /// re-run rather than refreshed, having no refresh token.
    pub fn is_client_credentials(self) -> bool {
        matches!(self, Self::ClientCredentials | Self::ClientCredentialsJwt)
    }
}

/// Endpoints of the OAuth 2.0 authorization server.
///
/// All optional at parse time: each command checks the ones it needs,
/// `auth get` the configured grant's, `token refresh` the token one,
/// `token show` none at all.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EndpointsConfig {
    /// Where the authorization code grant sends the user's browser.
    pub authorization: Option<Url>,
    /// Device authorization endpoint of `grant = "device"` (RFC 8628).
    pub device_authorization: Option<Url>,
    /// Where grants and refreshes exchange for a token.
    pub token: Option<Url>,
    /// Where the provider sends the browser back, a random
    /// `http://127.0.0.1:<port>` being bound when omitted.
    pub redirection: Option<Url>,
}

/// PKCE posture of the authorization code grant.
///
/// Accepts both TOML shapes, a boolean (true = S256, false = off) and a
/// method string ("s256" or "plain"). It defaults to S256, aligning
/// with OAuth 2.1, and the device grant ignores it.
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

/// The `storage` block, holding how the token is persisted.
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

/// System notification content, `$VAR` references being expanded from
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

/// Skips a field equal to its type's default, so a wizard-generated
/// configuration omits defaulted scalars.
pub(crate) fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// TLS provider selector, folded into the pimalaya-stream config.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TlsConfig {
    /// The provider the binary was built with.
    #[default]
    Auto,
    /// The platform-backed native-tls provider.
    NativeTls,
    /// The rustls provider on the aws-lc-rs crypto backend.
    RustlsAws,
    /// The rustls provider on the ring crypto backend.
    RustlsRing,
}

impl TlsConfig {
    /// Builds the runtime [`Tls`] handle the connect helpers expect,
    /// folding in the account's ALPN list.
    ///
    /// That list is the only way ALPN is set, so no call site can
    /// negotiate one by accident; an empty one skips the extension.
    pub fn into_tls(self, alpn: Vec<String>) -> Tls {
        let (provider, crypto) = match self {
            Self::Auto => (None, None),
            Self::NativeTls => (Some(TlsProvider::NativeTls), None),
            Self::RustlsAws => (Some(TlsProvider::Rustls), Some(RustlsCrypto::Aws)),
            Self::RustlsRing => (Some(TlsProvider::Rustls), Some(RustlsCrypto::Ring)),
        };

        Tls {
            provider,
            rustls: Rustls { crypto, alpn },
            cert: None,
        }
    }
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
    fn tls_and_alpn_fold_into_one_handle() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
tls = "rustls-aws"
alpn = ["http/1.1"]
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#,
        );

        let tls = account.tls.into_tls(account.alpn);

        assert!(matches!(tls.provider, Some(TlsProvider::Rustls)));
        assert!(matches!(tls.rustls.crypto, Some(RustlsCrypto::Aws)));
        assert_eq!(tls.rustls.alpn, ["http/1.1"]);
    }

    #[test]
    fn an_account_naming_neither_offers_no_alpn() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#,
        );

        assert_eq!(account.tls, TlsConfig::Auto);

        let tls = account.tls.into_tls(account.alpn);

        assert!(tls.provider.is_none());
        assert!(tls.rustls.alpn.is_empty());
    }

    #[test]
    fn interactive_grants_are_not_client_credentials() {
        assert!(!GrantConfig::AuthorizationCode.is_client_credentials());
        assert!(!GrantConfig::Device.is_client_credentials());
    }
}
