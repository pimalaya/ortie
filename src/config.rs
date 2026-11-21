//! # Configuration
//!
//! The TOML configuration of the `ortie` CLI. Every type here is a pure
//! DTO mirroring the nested shape (`storage.read.command`, ...), the
//! sole behaviour being [`TlsConfig::into_tls`], which folds the
//! selector and the ALPN list into the handle the connect helpers
//! expect. The enums also serialize as the configuration spells them,
//! which `account list` and `account show` print back.
//!
//! The flat runtime view commands consume is
//! [`crate::account::Account`], which this one is flattened into once
//! the selected account is taken.
//!
//! Loaded from every existing path among
//! $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml
//! and $HOME/.ortierc, each deep-merged on top of the ones before.
//! `-c, --config <PATH>` replaces them, repeated once per file: the first
//! is the base, the rest deep-merge on top.

use std::{collections::HashMap, fmt, path::PathBuf};

use pimalaya_config::{command::CommandConfig, secret::Secret, toml, toml::TomlConfig};
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsClient, TlsProvider, TlsServer};
use schemars::JsonSchema;
use serde::{
    Deserialize, Deserializer, Serialize,
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

    fn sample_url() -> &'static str {
        "https://github.com/pimalaya/ortie/blob/master/config.sample.toml"
    }

    fn accounts(&self) -> impl Iterator<Item = (&str, bool)> {
        let accounts = self.accounts.iter();
        accounts.map(|(name, account)| (name.as_str(), account.default))
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
    #[serde(default, alias = "client-certificate")]
    #[serde(deserialize_with = "toml::opt_shell_expanded_path")]
    pub client_cert: Option<PathBuf>,
    /// OAuth 2.0 grant flow run by the auth commands.
    #[serde(default)]
    pub grant: GrantConfig,
    /// Endpoints of the OAuth 2.0 authorization server.
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    /// TLS options of the HTTPS connections.
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
    /// RFC 8707 resource indicators the token is meant for, sent on the
    /// authorization request and the code exchange.
    #[serde(default)]
    pub resources: Vec<Url>,
    /// PKCE posture of the authorization code grant.
    #[serde(default)]
    pub pkce: PkceConfig,
    /// Extra parameters forwarded verbatim to the authorization request
    /// query, keyed by wire name and never kebab-renamed.
    #[serde(default)]
    pub extras: HashMap<String, String>,
    /// Whether `token show` refreshes an expired token by itself, on
    /// unless set to false.
    #[serde(default = "auto_refresh_default")]
    pub auto_refresh: bool,
    /// Shell commands reading and writing the persisted token.
    pub storage: StoragesConfig,
}

/// The default of [`AccountConfig::auto_refresh`]: on, since a caller of
/// `token show` wants a token it can use.
fn auto_refresh_default() -> bool {
    true
}

/// OAuth 2.0 grant flow run by the auth commands.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
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

    /// The grant as the configuration spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationCode => "authorization-code",
            Self::Device => "device",
            Self::ClientCredentials => "client-credentials",
            Self::ClientCredentialsJwt => "client-credentials-jwt",
        }
    }
}

/// Endpoints of the OAuth 2.0 authorization server.
///
/// All optional at parse time: each command checks the ones it needs,
/// `auth get` the configured grant's, `token refresh` the token one,
/// `token show` none until it refreshes.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EndpointsConfig {
    /// Where the authorization code grant sends the user's browser.
    pub authorization: Option<Url>,
    /// Device authorization endpoint of `grant = "device"` (RFC 8628).
    pub device_authorization: Option<Url>,
    /// Where grants and refreshes exchange for a token.
    pub token: Option<Url>,
    /// Where `token revoke` revokes the stored token (RFC 7009).
    pub revocation: Option<Url>,
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
    #[serde(alias = "cmd")]
    pub command: CommandConfig,
}

/// The `tls` table, the TLS options every Pimalaya CLI shares.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    /// The TLS implementation, the build's first when omitted.
    pub provider: Option<TlsProviderConfig>,
    /// Rustls-only options.
    #[serde(default)]
    pub rustls: RustlsConfig,
    /// A PEM certificate to trust beside the platform store.
    #[serde(default, deserialize_with = "toml::opt_shell_expanded_path")]
    pub cert: Option<PathBuf>,
    /// A client certificate, presented when the server asks for one.
    pub client: Option<TlsClientConfig>,
}

/// A client certificate and its private key, both PEM.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsClientConfig {
    /// The certificate, optionally followed by its intermediates.
    #[serde(deserialize_with = "toml::shell_expanded_path")]
    pub cert: PathBuf,
    /// The private key, PKCS#8 under native-tls.
    #[serde(deserialize_with = "toml::shell_expanded_path")]
    pub key: PathBuf,
}

/// The TLS implementation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TlsProviderConfig {
    /// The pure-Rust rustls stack.
    Rustls,
    /// The platform's own TLS stack, through native-tls.
    NativeTls,
}

/// Rustls-only options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    /// The crypto provider rustls runs on, the build's first when omitted.
    pub crypto: Option<RustlsCryptoConfig>,
}

/// The crypto provider rustls runs on.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustlsCryptoConfig {
    /// aws-lc-rs.
    Aws,
    /// ring.
    Ring,
}

impl TlsConfig {
    /// Builds the runtime [`Tls`] handle the connect helpers expect,
    /// folding in the account's ALPN list.
    ///
    /// That list is the only way ALPN is set, so no call site can
    /// negotiate one by accident; an empty one skips the extension.
    pub fn into_tls(self, alpn: Vec<String>) -> Tls {
        Tls {
            provider: self.provider.map(|provider| match provider {
                TlsProviderConfig::Rustls => TlsProvider::Rustls,
                TlsProviderConfig::NativeTls => TlsProvider::NativeTls,
            }),
            rustls: Rustls {
                crypto: self.rustls.crypto.map(|crypto| match crypto {
                    RustlsCryptoConfig::Aws => RustlsCrypto::Aws,
                    RustlsCryptoConfig::Ring => RustlsCrypto::Ring,
                }),
                alpn,
            },
            server: self.cert.map(|cert| TlsServer { cert }),
            client: self
                .client
                .map(|TlsClientConfig { cert, key }| TlsClient { cert, key }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::PathBuf};

    use anyhow::Result;
    use pimalaya_config::command::CommandConfig;
    use pimalaya_config::{sample::variants, toml::TomlConfig};
    use pimalaya_stream::tls::{RustlsCrypto, TlsProvider};

    use crate::config::{AccountConfig, Config, GrantConfig};

    fn try_parse(toml: &str) -> Result<Config> {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(toml.as_bytes()).unwrap();

        Config::from_paths(&[file.path().to_path_buf()])
    }

    fn parse(toml: &str) -> AccountConfig {
        let mut config = try_parse(toml).unwrap();
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
            account.client_cert,
            Some(PathBuf::from("/etc/ortie/cert.pem"))
        );
    }

    #[test]
    fn auto_refresh_is_on_unless_turned_off() {
        let account = |line: &str| {
            parse(&format!(
                r#"
[accounts.test]
client-id = "app-id"
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
{line}
"#
            ))
        };

        assert!(account("").auto_refresh);
        assert!(!account("auto-refresh = false").auto_refresh);
    }

    #[test]
    fn tls_and_alpn_fold_into_one_handle() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
tls.provider = "rustls"
tls.rustls.crypto = "aws"
tls.cert = "/etc/ortie/ca.pem"
alpn = ["http/1.1"]
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#,
        );

        let tls = account.tls.into_tls(account.alpn);

        assert!(matches!(tls.provider, Some(TlsProvider::Rustls)));
        assert!(matches!(tls.rustls.crypto, Some(RustlsCrypto::Aws)));
        assert_eq!(tls.rustls.alpn, ["http/1.1"]);

        let server = tls.server.map(|server| server.cert);
        assert_eq!(server, Some(PathBuf::from("/etc/ortie/ca.pem")));
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

        let tls = account.tls.into_tls(account.alpn);

        assert!(tls.provider.is_none());
        assert!(tls.rustls.crypto.is_none());
        assert!(tls.server.is_none());
        assert!(tls.rustls.alpn.is_empty());
    }

    /// The `tls` string Ortie read until 3.0.0 is refused by serde as any
    /// value of the wrong type, never by name.
    #[test]
    fn the_tls_string_is_refused_as_a_wrong_type() {
        for value in ["auto", "native-tls", "rustls-aws", "rustls-ring"] {
            let extra = format!("tls = \"{value}\"");
            let err = try_parse(&account_with("", &extra)).unwrap_err();
            let err = format!("{err:?}");
            assert!(err.contains("invalid type: string"), "{value}: {err}");
            assert!(!err.contains("MIGRATION.md"), "{value}: {err}");
        }

        let err = try_parse(&account_with("", "tls.typo = 1")).unwrap_err();
        assert!(
            format!("{err:?}").contains("unknown field `typo`"),
            "{err:?}"
        );
    }

    #[test]
    fn a_tls_client_certificate_reaches_the_handle_with_its_key() {
        let extra = "tls.client.cert = \"/etc/ortie/me.pem\"\n\
                     tls.client.key = \"/etc/ortie/me.key.pem\"";
        let mut config = try_parse(&account_with("", extra)).unwrap();
        let account = config.take_named_account("test").unwrap().1;

        let client = account.tls.into_tls(account.alpn).client.unwrap();
        assert_eq!(client.cert, PathBuf::from("/etc/ortie/me.pem"));
        assert_eq!(client.key, PathBuf::from("/etc/ortie/me.key.pem"));

        let extra = "tls.client.cert = \"/etc/ortie/me.pem\"";
        let err = try_parse(&account_with("", extra)).unwrap_err();
        assert!(
            format!("{err:?}").contains("missing field `key`"),
            "{err:?}"
        );

        let extra = "tls.client.typo = 1";
        let err = try_parse(&account_with("", extra)).unwrap_err();
        assert!(
            format!("{err:?}").contains("unknown field `typo`"),
            "{err:?}"
        );
    }

    /// `cmd` is the family alias of `command`, kept as written.
    #[test]
    fn a_storage_command_takes_the_cmd_alias() {
        let account = parse(
            r#"
[accounts.test]
client-id = "app-id"
storage.read.cmd = "pass show ortie"
storage.write.command = ["tee", "token.json"]
"#,
        );

        let read = CommandConfig::Shell(String::from("pass show ortie"));
        assert_eq!(account.storage.read.command, read);

        let write = CommandConfig::Argv {
            program: String::from("tee"),
            args: vec![String::from("token.json")],
        };
        assert_eq!(account.storage.write.command, write);
    }

    /// url's IDNA runs without a Unicode back end, so a host is ASCII and
    /// an internationalised one is written in its Punycode form.
    #[test]
    fn a_non_ascii_host_is_refused() {
        let account = |host: &str| {
            format!(
                r#"
[accounts.test]
client-id = "app-id"
grant = "client-credentials"
endpoints.token = "https://{host}/token"
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
"#
            )
        };

        assert!(try_parse(&account("bücher.example")).is_err());
        assert!(try_parse(&account("xn--bcher-kva.example")).is_ok());
    }

    /// An account every level of which is valid, `extra` lines appended.
    fn account_with(root: &str, extra: &str) -> String {
        format!(
            r#"
{root}
[accounts.test]
client-id = "app-id"
client-secret.raw = "s3cret"
endpoints.token = "https://login.example.com/token"
storage.read.command = ["cat", "token.json"]
storage.write.command = ["tee", "token.json"]
{extra}
"#
        )
    }

    #[test]
    fn an_unknown_key_is_refused_at_every_level() {
        let cases = [
            ("typo = 1", "", "typo"),
            ("", "client-idd = \"app-id\"", "client-idd"),
            ("", "endpoints.tokn = \"https://x.example\"", "tokn"),
            ("", "storage.typo = 1", "typo"),
            ("", "storage.read.typo = 1", "typo"),
            ("", "storage.write.typo = 1", "typo"),
        ];

        assert!(try_parse(&account_with("", "")).is_ok());

        for (root, extra, key) in cases {
            let err = try_parse(&account_with(root, extra)).unwrap_err();
            let err = format!("{err:?}");
            let needle = format!("unknown field `{key}`");
            assert!(err.contains(&needle), "{root}{extra}: {err}");
        }

        // pimalaya-config's secret is a one-key enum, refusing a second key
        // without naming it.
        assert!(try_parse(&account_with("", "client-secret.typo = 1")).is_err());
    }

    /// The `hooks` table, which Ortie read until 3.0.0, is an unknown key
    /// like any other, whatever it holds.
    #[test]
    fn a_removed_key_is_an_unknown_field() {
        let cases = [
            "hooks.on-issue.success.command = \"true\"",
            "hooks.on-refresh.error.notify.summary = \"Ortie\"",
            "hooks = {}",
            "[accounts.test.hooks.on-issue.success]\ncommand = \"true\"",
        ];

        for extra in cases {
            let err = try_parse(&account_with("", extra)).unwrap_err();
            let err = format!("{err:?}");
            assert!(err.contains("unknown field `hooks`"), "{extra}: {err}");
            assert!(!err.contains("MIGRATION.md"), "{extra}: {err}");
        }
    }

    #[test]
    fn interactive_grants_are_not_client_credentials() {
        assert!(!GrantConfig::AuthorizationCode.is_client_credentials());
        assert!(!GrantConfig::Device.is_client_credentials());
    }

    /// The sections of config.sample.toml a user uncomments, the
    /// providers' accounts living in the README.
    const SECTIONS: [&str; 1] = ["# Custom"];

    /// Checks config.sample.toml: one `[accounts.example]` table and no
    /// other, its custom section parsed through the loader `-c` uses with
    /// every variant on top.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let text = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = text.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        for variant in variants(&text, &SECTIONS) {
            let name = variant.name;
            let config = try_parse(&variant.text).unwrap_or_else(|err| {
                panic!("config.sample.toml with {name} uncommented: {err:?}")
            });
            assert!(config.accounts.contains_key("example"), "{name}");
        }
    }

    /// The fenced toml blocks of `markdown`, each with the line number of
    /// its fence.
    fn toml_blocks(markdown: &str) -> Vec<(usize, String)> {
        let mut blocks: Vec<(usize, String)> = Vec::new();
        let mut open = false;

        for (number, line) in markdown.lines().enumerate() {
            if line == "```toml" {
                blocks.push((number + 1, String::new()));
                open = true;
            } else if line == "```" {
                open = false;
            } else if open {
                let block = &mut blocks.last_mut().unwrap().1;
                block.push_str(line);
                block.push('\n');
            }
        }

        blocks
    }

    /// Checks the provider accounts of README.md: each toml block one or
    /// more `[accounts.<name>]` tables, parsed through the loader `-c`
    /// uses and holding those accounts, the live suites reading them by
    /// name.
    #[test]
    fn the_readme_provider_accounts_parse() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let readme = fs::read_to_string(&path).unwrap();
        let mut names = Vec::new();

        for (line, block) in toml_blocks(&readme) {
            let config =
                try_parse(&block).unwrap_or_else(|err| panic!("README.md:{line}: {err:?}"));
            let headers: Vec<_> = block.lines().filter(|l| l.starts_with('[')).collect();
            assert!(!headers.is_empty(), "README.md:{line}: no table");

            for header in headers {
                let name = header
                    .strip_prefix("[accounts.")
                    .and_then(|name| name.strip_suffix(']'))
                    .unwrap_or_else(|| panic!("README.md:{line}: {header}"));
                assert!(config.accounts.contains_key(name), "README.md:{line}");
                names.push(name.to_owned());
            }
        }

        let expected = [
            "google",
            "microsoft-graph",
            "microsoft-imap-smtp",
            "fastmail",
        ];
        assert_eq!(names, expected);
    }
}
