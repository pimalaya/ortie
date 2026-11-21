//! # Account show command
//!
//! The `account show` subcommand, printing every property of one account
//! as the configuration loads it, one field per configuration key, its
//! raw secrets redacted.

use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use pimalaya_config::{command::CommandConfig, secret::Secret, toml::TomlConfig};
use schemars::JsonSchema;
use serde::Serialize;

use crate::config::{
    AccountConfig, Config, GrantConfig, PkceConfig, RustlsCryptoConfig, TlsProviderConfig,
};

/// What a raw secret prints as in the text output.
const REDACTED: &str = "<redacted>";

/// Show every property of the account.
///
/// Prints the account as the configuration loads it: every `-c` layer
/// merged, every default filled in, the optional properties it leaves out
/// left out. Nothing is reached: no storage command runs and no request
/// is sent. Exits 0, or 1 on an account it cannot take.
///
/// The text output is `Account: <name>` then one aligned `<key>: <value>`
/// field per property, labelled by its configuration key
/// (`client-id`, `endpoints.token`, `storage.read.command`), a list one
/// value per line, a command as a shell would read it. A raw secret is
/// never printed: `client-secret.raw: <redacted>` stands for it.
///
/// JSON output: `{"account": {"name", "default", "clientId",
/// "clientSecret", "clientKey", "clientCert", "grant", "endpoints",
/// "tls", "alpn", "scopes", "resources", "pkce", "extras", "autoRefresh",
/// "storage"}}`, the keys camelCase, an optional one left out when
/// unset, `tls` too when no key of it is set, a raw `clientSecret`
/// printing `{"raw": null, "redacted": true}` and a command one
/// `{"command": ...}`.
#[derive(Debug, Parser)]
pub struct AccountShowCommand;

impl AccountShowCommand {
    /// Loads the configuration, takes the account `account_name` names
    /// (the default one when `None`) and prints it.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        let (name, config) = Config::load(config_paths)?.take_account(account_name)?;
        let account = AccountShow::new(name, &config);
        printer.out(AccountShowOutput { account })
    }
}

/// The `account show` output, one account.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowOutput {
    /// The account as loaded.
    pub account: AccountShow,
}

/// Every property of one account, as loaded.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShow {
    /// The name of its `[accounts.<name>]` block.
    pub name: String,
    /// Whether a command with no `-a <NAME>` runs against it.
    pub default: bool,
    /// The OAuth 2.0 client identifier.
    pub client_id: String,
    /// The client secret, left out when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<AccountShowSecret>,
    /// The private key signing the JWT assertions, left out when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
    /// The certificate of the JWT assertions, left out when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,
    /// The OAuth 2.0 grant the auth commands run.
    pub grant: GrantConfig,
    /// The endpoints of the authorization server.
    pub endpoints: AccountShowEndpoints,
    /// The TLS options of the HTTPS connections, left out when the account
    /// sets none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<AccountShowTls>,
    /// The ALPN identifiers offered during the TLS handshake.
    pub alpn: Vec<String>,
    /// The scopes requested for the access token.
    pub scopes: Vec<String>,
    /// The RFC 8707 resource indicators the token is meant for.
    pub resources: Vec<String>,
    /// The PKCE posture of the authorization code grant: `s256`, `plain`,
    /// or `false` when off.
    pub pkce: AccountShowPkce,
    /// The extra parameters of the authorization request, by wire name.
    pub extras: BTreeMap<String, String>,
    /// Whether `token show` refreshes an expired token by itself.
    pub auto_refresh: bool,
    /// The commands reading and writing the token.
    pub storage: AccountShowStorages,
}

impl AccountShow {
    /// The properties of the account `name` loaded as `config`.
    pub fn new(name: String, config: &AccountConfig) -> Self {
        let endpoints = &config.endpoints;
        let url = |url: &Option<url::Url>| url.as_ref().map(|url| url.to_string());

        Self {
            name,
            default: config.default,
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.as_ref().map(|secret| match secret {
                Secret::Raw(_) => AccountShowSecret::Raw(AccountShowRedacted {
                    raw: (),
                    redacted: true,
                }),
                Secret::Command(command) => AccountShowSecret::Command {
                    command: command.into(),
                },
            }),
            client_key: config.client_key.as_deref().map(path),
            client_cert: config.client_cert.as_deref().map(path),
            grant: config.grant,
            endpoints: AccountShowEndpoints {
                authorization: url(&endpoints.authorization),
                device_authorization: url(&endpoints.device_authorization),
                token: url(&endpoints.token),
                revocation: url(&endpoints.revocation),
                redirection: url(&endpoints.redirection),
            },
            tls: Some(AccountShowTls {
                provider: config.tls.provider,
                rustls: AccountShowRustls {
                    crypto: config.tls.rustls.crypto,
                },
                cert: config.tls.cert.as_deref().map(path),
                client: config
                    .tls
                    .client
                    .as_ref()
                    .map(|client| AccountShowTlsClient {
                        cert: path(&client.cert),
                        key: path(&client.key),
                    }),
            })
            .filter(|tls| {
                tls.provider.is_some()
                    || tls.rustls.crypto.is_some()
                    || tls.cert.is_some()
                    || tls.client.is_some()
            }),
            alpn: config.alpn.clone(),
            scopes: config.scopes.clone(),
            resources: config.resources.iter().map(ToString::to_string).collect(),
            pkce: match config.pkce {
                PkceConfig::S256 => AccountShowPkce::Method(AccountShowPkceMethod::S256),
                PkceConfig::Plain => AccountShowPkce::Method(AccountShowPkceMethod::Plain),
                PkceConfig::Off => AccountShowPkce::Off(false),
            },
            extras: config.extras.clone().into_iter().collect(),
            auto_refresh: config.auto_refresh,
            storage: AccountShowStorages {
                read: AccountShowStorage {
                    command: (&config.storage.read.command).into(),
                },
                write: AccountShowStorage {
                    command: (&config.storage.write.command).into(),
                },
            },
        }
    }
}

/// A secret of the account: a raw one redacted, a command as written.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AccountShowSecret {
    /// A raw secret, its value never printed.
    Raw(AccountShowRedacted),
    /// A command printing the secret.
    Command {
        /// The command, as written.
        command: AccountShowCommandLine,
    },
}

/// A raw secret in place of its value.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowRedacted {
    /// Always `null`: the value is never printed.
    pub raw: (),
    /// Always true.
    pub redacted: bool,
}

/// A command, a shell line or a program and its arguments.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AccountShowCommandLine {
    /// A shell line.
    Shell(String),
    /// A program then its arguments.
    Argv(Vec<String>),
}

impl From<&CommandConfig> for AccountShowCommandLine {
    fn from(command: &CommandConfig) -> Self {
        match command {
            CommandConfig::Shell(line) => Self::Shell(line.clone()),
            CommandConfig::Argv { program, args } => {
                Self::Argv([program].into_iter().chain(args).cloned().collect())
            }
        }
    }
}

/// The endpoints of the authorization server, each left out when unset.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowEndpoints {
    /// The authorization endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    /// The device authorization endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_authorization: Option<String>,
    /// The token endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The revocation endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<String>,
    /// The redirection endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirection: Option<String>,
}

/// The TLS options, each left out when unset.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowTls {
    /// The TLS implementation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<TlsProviderConfig>,
    /// The rustls-only options.
    pub rustls: AccountShowRustls,
    /// The PEM certificate trusted beside the platform store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,
    /// The client certificate and its key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<AccountShowTlsClient>,
}

/// The rustls-only options.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowRustls {
    /// The crypto provider, left out when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto: Option<RustlsCryptoConfig>,
}

/// A client certificate and its private key.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowTlsClient {
    /// The certificate path.
    pub cert: String,
    /// The private key path.
    pub key: String,
}

/// The PKCE posture, as the configuration writes it.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AccountShowPkce {
    /// A code challenge method.
    Method(AccountShowPkceMethod),
    /// PKCE off, always `false`.
    Off(bool),
}

/// A PKCE code challenge method.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountShowPkceMethod {
    /// SHA-256.
    S256,
    /// Plain.
    Plain,
}

/// The commands reading and writing the token.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowStorages {
    /// The command printing the stored token.
    pub read: AccountShowStorage,
    /// The command receiving the token.
    pub write: AccountShowStorage,
}

/// One storage direction.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountShowStorage {
    /// The command, as written.
    pub command: AccountShowCommandLine,
}

/// The account as fields: `Account` then one per property, labelled by
/// its configuration key, a list one value per line, every value
/// sanitized.
impl fmt::Display for AccountShowOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let account = &self.account;
        let endpoints = &account.endpoints;
        let mut rows: Vec<(String, String)> = Vec::new();
        let mut push = |label: &str, value: &str| rows.push((label.into(), text(value)));

        push("Account", &account.name);
        push("default", &account.default.to_string());
        push("client-id", &account.client_id);

        match &account.client_secret {
            Some(AccountShowSecret::Raw(_)) => push("client-secret.raw", REDACTED),
            Some(AccountShowSecret::Command { command }) => {
                push("client-secret.command", &command.to_string())
            }
            None => (),
        }

        let optionals = [
            ("client-key", &account.client_key),
            ("client-cert", &account.client_cert),
            ("grant", &Some(account.grant.as_str().into())),
            ("endpoints.authorization", &endpoints.authorization),
            (
                "endpoints.device-authorization",
                &endpoints.device_authorization,
            ),
            ("endpoints.token", &endpoints.token),
            ("endpoints.revocation", &endpoints.revocation),
            ("endpoints.redirection", &endpoints.redirection),
        ];

        for (label, value) in optionals {
            if let Some(value) = value {
                push(label, value);
            }
        }

        if let Some(tls) = &account.tls {
            if let Some(provider) = tls.provider {
                let provider = match provider {
                    TlsProviderConfig::Rustls => "rustls",
                    TlsProviderConfig::NativeTls => "native-tls",
                };
                push("tls.provider", provider);
            }

            if let Some(crypto) = tls.rustls.crypto {
                let crypto = match crypto {
                    RustlsCryptoConfig::Aws => "aws",
                    RustlsCryptoConfig::Ring => "ring",
                };
                push("tls.rustls.crypto", crypto);
            }

            if let Some(cert) = &tls.cert {
                push("tls.cert", cert);
            }

            if let Some(client) = &tls.client {
                push("tls.client.cert", &client.cert);
                push("tls.client.key", &client.key);
            }
        }

        let lists = [
            ("alpn", &account.alpn),
            ("scopes", &account.scopes),
            ("resources", &account.resources),
        ];

        for (label, items) in lists {
            let items: Vec<String> = items.iter().map(|item| text(item)).collect();
            rows.push((label.into(), items.join("\n")));
        }

        let mut push = |label: &str, value: &str| rows.push((label.into(), text(value)));

        let pkce = match account.pkce {
            AccountShowPkce::Method(AccountShowPkceMethod::S256) => "s256",
            AccountShowPkce::Method(AccountShowPkceMethod::Plain) => "plain",
            AccountShowPkce::Off(_) => "false",
        };
        push("pkce", pkce);

        if account.extras.is_empty() {
            push("extras", "");
        }

        for (key, value) in &account.extras {
            push(&format!("extras.{}", extra_key(key)), value);
        }

        push("auto-refresh", &account.auto_refresh.to_string());

        let storage = &account.storage;
        push("storage.read.command", &storage.read.command.to_string());
        push("storage.write.command", &storage.write.command.to_string());

        let mut fields = Fields::default();

        for (label, value) in &rows {
            fields.push(label, value);
        }

        write!(f, "{fields}")
    }
}

/// The command as a shell would read it, as pimalaya-config writes a
/// [`CommandConfig`].
impl fmt::Display for AccountShowCommandLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = match self {
            Self::Shell(line) => CommandConfig::Shell(line.clone()),
            Self::Argv(argv) => CommandConfig::Argv {
                program: argv.first().cloned().unwrap_or_default(),
                args: argv.iter().skip(1).cloned().collect(),
            },
        };

        write!(f, "{command}")
    }
}

/// `path` as printed, lossy where it is no UTF-8.
fn path(path: &Path) -> String {
    path.display().to_string()
}

/// `s` with its control and bidi characters replaced, as the kit's text
/// outputs print a value.
fn text(s: &str) -> String {
    sanitize::text(s).into_owned()
}

/// An `extras` key as the configuration spells it: bare when TOML takes
/// it bare, quoted otherwise, sanitized either way.
fn extra_key(key: &str) -> String {
    let bare = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';

    if !key.is_empty() && key.chars().all(bare) {
        String::from(key)
    } else {
        format!("\"{}\"", text(key))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use pimalaya_config::{secret::Secret, toml::TomlConfig};
    use secrecy::ExposeSecret;

    use crate::{
        account::show::{AccountShow, AccountShowOutput},
        config::{AccountConfig, Config},
    };

    /// The account `name` of the configuration `toml`, as the loader
    /// `-c` uses reads it.
    fn load(toml: &str, name: &str) -> AccountConfig {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(toml.as_bytes()).unwrap();

        let mut config = Config::from_paths(&[file.path().to_path_buf()]).unwrap();
        config.take_named_account(name).unwrap().1
    }

    /// The text output prints one field per property, labelled by its
    /// configuration key: no raw secret, and no value, a line feed
    /// included, that breaks a line or drives the terminal.
    #[test]
    fn the_text_output_is_one_field_per_key() {
        let toml = r#"
[accounts."my work"]
default = true
client-id = "c"
client-secret.raw = "s3cret"
client-key = "/etc/ortie/key.pem"
client-cert = "/etc/ortie/cert.pem"
grant = "device"
endpoints.authorization = "https://example.com/authorize"
endpoints.device-authorization = "https://example.com/device"
endpoints.token = "https://example.com/token?tenant=x"
endpoints.revocation = "https://example.com/revoke"
endpoints.redirection = "http://localhost"
tls.provider = "native-tls"
tls.rustls.crypto = "ring"
tls.cert = "/etc/ortie/ca.pem"
tls.client.cert = "/etc/ortie/me.pem"
tls.client.key = "/etc/ortie/me.key.pem"
alpn = ["http/1.1"]
scopes = ["openid", "a \"quoted\\\" \u202E\u0007 scope\nalpn: forged"]
resources = ["https://example.com/jmap"]
pkce = "plain"
extras.access_type = "offline"
extras."odd key" = "tab\there"
auto-refresh = false
storage.read.command = ["cat", "/tmp/token.json"]
storage.write.command = "tee /tmp/token.json"
"#;

        let original = load(toml, "my work");
        let shown = AccountShow::new(String::from("my work"), &original);
        let text = AccountShowOutput {
            account: shown.clone(),
        }
        .to_string();

        assert!(!text.contains("s3cret"), "{text}");
        assert!(!text.contains('\u{202e}'), "{text}");
        assert!(!text.contains('\u{7}'), "{text}");
        assert!(!text.contains("\nalpn: forged"), "{text}");

        let labels: Vec<&str> = text
            .lines()
            .filter_map(|line| line.split_once(": ").map(|(label, _)| label))
            .filter(|label| !label.starts_with(' '))
            .collect();
        let expected = [
            "Account",
            "default",
            "client-id",
            "client-secret.raw",
            "client-key",
            "client-cert",
            "grant",
            "endpoints.authorization",
            "endpoints.device-authorization",
            "endpoints.token",
            "endpoints.revocation",
            "endpoints.redirection",
            "tls.provider",
            "tls.rustls.crypto",
            "tls.cert",
            "tls.client.cert",
            "tls.client.key",
            "alpn",
            "scopes",
            "resources",
            "pkce",
            "extras.access_type",
            "extras.\"odd key\"",
            "auto-refresh",
            "storage.read.command",
            "storage.write.command",
        ];
        assert_eq!(labels, expected, "{text}");

        let line = "\nclient-secret.raw:               <redacted>\n";
        assert!(text.contains(line), "{text}");
        let line = "\nstorage.read.command:            cat /tmp/token.json\n";
        assert!(text.contains(line), "{text}");

        match original.client_secret {
            Some(Secret::Raw(secret)) => assert_eq!(secret.expose_secret(), "s3cret"),
            secret => panic!("{secret:?}"),
        }
    }

    /// The JSON output carries no raw secret value, whatever it is.
    #[test]
    fn the_json_output_redacts_a_raw_secret() {
        let toml = r#"
[accounts.a]
client-id = "c"
client-secret.raw = "s3cret"
storage.read.command = "true"
storage.write.command = "true"
"#;

        let account = AccountShow::new(String::from("a"), &load(toml, "a"));
        let json = serde_json::to_value(AccountShowOutput { account }).unwrap();

        let redacted = serde_json::json!({ "raw": null, "redacted": true });
        assert_eq!(json["account"]["clientSecret"], redacted);
        assert!(!json.to_string().contains("s3cret"), "{json}");
    }

    /// The JSON output leaves `tls` out when the account sets no key of
    /// it, and prints it once one is set.
    #[test]
    fn the_json_output_leaves_out_an_unset_tls() {
        let toml = r#"
[accounts.a]
client-id = "c"
storage.read.command = "true"
storage.write.command = "true"

[accounts.b]
client-id = "c"
tls.cert = "/etc/ortie/ca.pem"
storage.read.command = "true"
storage.write.command = "true"
"#;

        let account = AccountShow::new(String::from("a"), &load(toml, "a"));
        let json = serde_json::to_value(AccountShowOutput { account }).unwrap();
        assert_eq!(json["account"].get("tls"), None, "{json}");

        let account = AccountShow::new(String::from("b"), &load(toml, "b"));
        let json = serde_json::to_value(AccountShowOutput { account }).unwrap();
        let tls = serde_json::json!({ "rustls": {}, "cert": "/etc/ortie/ca.pem" });
        assert_eq!(json["account"]["tls"], tls, "{json}");
    }
}
