//! # Account
//!
//! The flat runtime account, built from the nested
//! [`crate::config::AccountConfig`] by flattening every
//! `storage.*.command` into a field.
//!
//! Commands consume it and call its driver methods rather than walking
//! the config tree, so token storage has one entry point.

use std::{
    borrow::Cow,
    collections::HashMap,
    io::Write,
    net::TcpListener,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use pimalaya_config::secret::Secret;
use pimalaya_stream::tls::Tls;
use url::{Host, Url};

use io_oauth::rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams;

use crate::{
    auth::get::is_loopback_redirect,
    config::{
        AccountConfig, EndpointsConfig, GrantConfig, PkceConfig, StorageConfig, StoragesConfig,
    },
};

/// The redirection bound when the account configures none.
pub const DEFAULT_REDIRECTION: &str = "http://127.0.0.1";

/// Flat, command-ready view of one OAuth 2.0 account.
#[derive(Debug)]
pub struct Account {
    /// OAuth 2.0 client identifier.
    pub client_id: String,
    /// Optional OAuth 2.0 client secret.
    pub client_secret: Option<Secret>,
    /// Private key signing the JWT assertions, re-read at every mint.
    pub client_key: Option<PathBuf>,
    /// Certificate deriving the assertion `x5t` thumbprint, recomputed
    /// at every mint.
    pub client_certificate: Option<PathBuf>,
    /// OAuth 2.0 grant flow run by the auth commands.
    pub grant: GrantConfig,
    /// TLS provider used for the HTTPS connections.
    pub tls: Tls,
    /// OAuth 2.0 scopes requested for the access token.
    pub scopes: Vec<String>,
    /// The RFC 8707 resource indicators the token is meant for.
    pub resources: Vec<Url>,
    /// PKCE posture of the authorization code grant.
    pub pkce: PkceConfig,
    /// Extra parameters forwarded verbatim to the authorization request
    /// query.
    pub extras: HashMap<String, String>,
    /// Whether `token show` refreshes an expired token by itself.
    pub auto_refresh: bool,
    /// Authorization endpoint of the authorization code grant.
    pub authorization_endpoint: Option<Url>,
    /// Device authorization endpoint of the device grant (RFC 8628).
    pub device_authorization_endpoint: Option<Url>,
    /// Token endpoint shared by grants and refreshes.
    pub token_endpoint: Option<Url>,
    /// The revocation endpoint of `token revoke` (RFC 7009).
    pub revocation_endpoint: Option<Url>,
    /// Redirection endpoint registered with the provider.
    pub redirection_endpoint: Option<Url>,
    /// Command printing the stored token JSON on its stdout.
    pub read_storage_command: Command,
    /// Command receiving the token JSON on its stdin.
    pub write_storage_command: Command,
    /// Token resolved from storage, memoized for the session.
    pub token: Option<Oauth20AccessTokenSuccessParams>,
}

impl From<AccountConfig> for Account {
    fn from(cfg: AccountConfig) -> Self {
        let AccountConfig {
            client_id,
            client_secret,
            client_key,
            client_cert: client_certificate,
            grant,
            endpoints,
            tls,
            alpn,
            scopes,
            resources,
            pkce,
            extras,
            auto_refresh,
            storage,
            ..
        } = cfg;

        let EndpointsConfig {
            authorization,
            device_authorization,
            token,
            revocation,
            redirection,
        } = endpoints;

        let StoragesConfig {
            read: StorageConfig { command: read_cmd },
            write: StorageConfig { command: write_cmd },
        } = storage;

        Self {
            client_id,
            client_secret,
            client_key,
            client_certificate,
            grant,
            tls: tls.into_tls(alpn),
            scopes,
            resources,
            pkce,
            extras,
            auto_refresh,
            authorization_endpoint: authorization,
            device_authorization_endpoint: device_authorization,
            token_endpoint: token,
            revocation_endpoint: revocation,
            redirection_endpoint: redirection,
            read_storage_command: read_cmd,
            write_storage_command: write_cmd,
            token: None,
        }
    }
}

impl Account {
    /// Resolves the redirection URI, binding an ephemeral port for
    /// `http://127.0.0.1` when none is configured, or for a configured
    /// http loopback URI naming no port.
    pub fn redirection(&self) -> Result<Cow<'_, Url>> {
        let url = match self.redirection_endpoint.as_ref() {
            Some(url) if url.scheme() != "http" || url.port().is_some() => {
                return Ok(Cow::Borrowed(url));
            }
            Some(url) if !is_loopback_redirect(url) => return Ok(Cow::Borrowed(url)),
            Some(url) => url.clone(),
            None => DEFAULT_REDIRECTION.parse()?,
        };

        let (url, _) = bind_loopback(&url)?;

        Ok(Cow::Owned(url))
    }

    /// Resolves the account token, memoizing it so the read storage
    /// command runs once per session.
    pub fn resolve_token(&mut self) -> Result<Oauth20AccessTokenSuccessParams> {
        if let Some(token) = &self.token {
            return Ok(token.clone());
        }

        let token = self.read_storage()?;
        self.token = Some(token.clone());
        Ok(token)
    }

    /// Runs the read storage command and parses its stdout as the token
    /// response JSON.
    fn read_storage(&mut self) -> Result<Oauth20AccessTokenSuccessParams> {
        let cmd = &mut self.read_storage_command;

        let output = cmd
            .output()
            .context("Spawn command to read OAuth 2.0 access token")?;

        if !output.status.success() {
            let bytes = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            let err = anyhow!("{}", String::from_utf8_lossy(&bytes));
            return Err(err.context("Read access token via command error"));
        }

        let res = Oauth20AccessTokenSuccessParams::try_from(output.stdout.as_slice())
            .context("Parse access token from command error")?;

        Ok(res)
    }

    /// Pipes the token response JSON to the write storage command.
    ///
    /// The local issuance time is stamped before the write, since no
    /// server reports one. The token is cached in memory before the
    /// write: a server rotating refresh tokens has invalidated the old
    /// one, so a failed write leaves a REPL session holding the new.
    pub fn write_to_storage(
        &mut self,
        mut res: Oauth20AccessTokenSuccessParams,
    ) -> Result<Oauth20AccessTokenSuccessParams> {
        res.issued_at = Some(now_secs());
        self.token = Some(res.clone());

        let cmd = &mut self.write_storage_command;
        let json = String::try_from(&res)?.into_bytes();

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Spawn command to save OAuth 2.0 access token")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&json)
                .context("Write access token to command stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Wait for save command to finish")?;

        if !output.status.success() {
            let err = "Write access token via command error";

            let data = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };

            if data.is_empty() {
                bail!(err);
            }

            let err2 = anyhow!("{}", String::from_utf8_lossy(&data));
            return Err(err2.context(err));
        }

        Ok(res)
    }
}

/// Binds a listener for the loopback redirection `uri`, on an ephemeral
/// port when an http `uri` names none (RFC 8252 section 7.3), and
/// returns `uri` carrying the bound port.
pub fn bind_loopback(uri: &Url) -> Result<(Url, TcpListener)> {
    let port = match uri.port() {
        Some(port) => port,
        None if uri.scheme() == "http" => 0,
        None => uri
            .port_or_known_default()
            .ok_or_else(|| anyhow!("Missing port in redirection {uri}"))?,
    };

    let listener = match uri.host() {
        Some(Host::Domain(domain)) => TcpListener::bind((domain, port)),
        Some(Host::Ipv4(ip)) => TcpListener::bind((ip, port)),
        Some(Host::Ipv6(ip)) => TcpListener::bind((ip, port)),
        None => bail!("Missing host in redirection {uri}"),
    }
    .with_context(|| format!("Bind the redirection listener {uri}"))?;

    let mut uri = uri.clone();

    if port == 0 {
        let port = listener.local_addr()?.port();

        uri.set_port(Some(port))
            .map_err(|()| anyhow!("Cannot set port {port} on redirection {uri}"))?;
    }

    Ok((uri, listener))
}

/// Current Unix epoch seconds, stamped as the local token issuance time.
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use crate::account::bind_loopback;

    #[test]
    fn a_port_less_http_loopback_binds_an_ephemeral_port() {
        let (uri, listener) = bind_loopback(&"http://localhost".parse().unwrap()).unwrap();
        let port = listener.local_addr().unwrap().port();

        assert_ne!(port, 80);
        assert_eq!(uri.as_str(), format!("http://localhost:{port}/"));
    }

    #[test]
    fn a_loopback_naming_its_port_keeps_it() {
        let free = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = free.local_addr().unwrap().port();
        drop(free);

        let uri = format!("http://127.0.0.1:{port}/cb").parse().unwrap();
        let (bound, _) = bind_loopback(&uri).unwrap();

        assert_eq!(bound, uri);
    }
}
