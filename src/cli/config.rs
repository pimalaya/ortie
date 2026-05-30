//! TOML configuration for the `ortie` CLI.
//!
//! All types here are pure DTOs: they mirror the nested TOML shape
//! (`storage.read.command`, `hooks.on-refresh.error.notify`, ...)
//! and carry no behaviour. The merged, flat runtime view that
//! commands consume lives in [`crate::cli::account::Account`].
//!
//! Loaded from the first valid path among:
//! - `$XDG_CONFIG_HOME/ortie/config.toml`
//! - `$HOME/.config/ortie/config.toml`
//! - `$HOME/.ortierc`
//!
//! Override with `-c, --config <PATH>` or `ORTIE_CONFIG=<PATH>`.

use alloc::{string::String, vec::Vec};
use std::{collections::HashMap, process::Command};

use pimalaya_config::{command, secret::Secret, toml::TomlConfig};
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
#[cfg(feature = "notify")]
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use url::Url;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
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
    #[serde(default)]
    pub default: bool,

    pub client_id: String,
    pub client_secret: Option<Secret>,

    pub endpoints: EndpointsConfig,
    #[serde(default, deserialize_with = "tls")]
    pub tls: Tls,

    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub pkce: bool,
    #[serde(default)]
    pub auto_refresh: bool,

    pub storage: StoragesConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EndpointsConfig {
    pub authorization: Url,
    pub token: Url,
    pub redirection: Option<Url>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct StoragesConfig {
    pub read: StorageConfig,
    pub write: StorageConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct StorageConfig {
    #[serde(alias = "cmd", with = "command")]
    pub command: Command,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HooksConfig {
    #[serde(default)]
    pub on_issue: HookStatusConfig,
    #[serde(default)]
    pub on_refresh: HookStatusConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HookStatusConfig {
    #[serde(default)]
    pub success: HookConfig,
    #[serde(default)]
    pub error: HookConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct HookConfig {
    #[serde(default, alias = "cmd", deserialize_with = "deserialize_opt_command")]
    pub command: Option<Command>,
    #[cfg(feature = "notify")]
    #[serde(default)]
    pub notify: Option<NotifyConfig>,
}

#[cfg(feature = "notify")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct NotifyConfig {
    pub summary: String,
    pub body: String,
}

#[cfg(not(feature = "notify"))]
pub type NotifyConfig = ();

fn deserialize_opt_command<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Command>, D::Error> {
    command::deserialize(de).map(Some)
}

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
