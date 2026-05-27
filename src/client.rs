//! Mid-level blocking OAuth 2.0 client.
//!
//! Drives the I/O-free coroutines under
//! [`crate::authorization_code_grant`], [`crate::issue_access_token`]
//! and [`crate::refresh_access_token`] against a single token
//! endpoint, using a [`StreamStd`] connection. Each method opens a
//! fresh stream, runs the coroutine loop inline, and returns the
//! parsed response.

use std::{
    borrow::Cow,
    io::{Read, Write},
};

use anyhow::{anyhow, bail, Result};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use io_http::rfc9110::request::HttpRequest;
use pimalaya_stream::{std::stream::StreamStd, tls::Tls};
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::{
    authorization_code_grant::access_token_request::{
        AccessTokenRequestParams, RequestOauth2AccessToken, RequestOauth2AccessTokenResult,
    },
    issue_access_token::AccessTokenResponse,
    refresh_access_token::{
        RefreshAccessTokenParams, RefreshOauth2AccessToken, RefreshOauth2AccessTokenResult,
    },
};

#[cfg(feature = "pkce")]
use crate::authorization_code_grant::pkce::PkceCodeVerifier;

const READ_BUFFER_SIZE: usize = 8 * 1024;

/// Blocking OAuth 2.0 client driving the I/O-free coroutines.
pub struct OauthClient<'a> {
    pub token_endpoint: &'a Url,
    pub tls: &'a Tls,
    pub client_id: &'a str,
    pub client_secret: Option<SecretString>,
}

impl<'a> OauthClient<'a> {
    pub fn new(token_endpoint: &'a Url, tls: &'a Tls, client_id: &'a str) -> Self {
        Self {
            token_endpoint,
            tls,
            client_id,
            client_secret: None,
        }
    }

    /// Exchanges an authorization code for an access token.
    pub fn request_access_token(
        &self,
        code: Cow<'_, str>,
        redirect_uri: Option<Cow<'_, str>>,
        #[cfg(feature = "pkce")] pkce_code_verifier: Option<Cow<'_, PkceCodeVerifier>>,
    ) -> Result<AccessTokenResponse> {
        let mut stream = self.connect()?;
        let request = self.request_builder()?;

        let params = AccessTokenRequestParams {
            code,
            redirect_uri,
            client_id: self.client_id.into(),
            #[cfg(feature = "pkce")]
            pkce_code_verifier,
        };

        let mut send = RequestOauth2AccessToken::new(request, params);
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match send.resume(arg.take()) {
                RequestOauth2AccessTokenResult::Ok(res) => return Ok(res),
                RequestOauth2AccessTokenResult::WantsRead => {
                    let n = stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                RequestOauth2AccessTokenResult::WantsWrite(bytes) => stream.write_all(&bytes)?,
                RequestOauth2AccessTokenResult::Err(err) => {
                    let ctx = "Request OAuth 2.0 access token error";
                    return Err(anyhow!("{err}").context(ctx));
                }
            }
        }
    }

    /// Refreshes an access token using a refresh token.
    pub fn refresh_access_token(
        &self,
        refresh_token: SecretString,
        scopes: impl IntoIterator<Item = String>,
    ) -> Result<AccessTokenResponse> {
        let mut stream = self.connect()?;
        let request = self.request_builder()?;

        let params = RefreshAccessTokenParams {
            client_id: self.client_id.to_string(),
            refresh_token,
            scopes: scopes.into_iter().map(Into::into).collect(),
        };

        let mut send = RefreshOauth2AccessToken::new(request, params);
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match send.resume(arg.take()) {
                RefreshOauth2AccessTokenResult::Ok(res) => return Ok(res),
                RefreshOauth2AccessTokenResult::WantsRead => {
                    let n = stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                RefreshOauth2AccessTokenResult::WantsWrite(bytes) => stream.write_all(&bytes)?,
                RefreshOauth2AccessTokenResult::Err(err) => {
                    let ctx = "Refresh OAuth 2.0 access token error";
                    return Err(anyhow!("{err}").context(ctx));
                }
            }
        }
    }

    fn connect(&self) -> Result<StreamStd> {
        let endpoint = self.token_endpoint;

        let Some(host) = endpoint.host_str() else {
            bail!("Missing token endpoint host name in {endpoint}");
        };

        let Some(port) = endpoint.port_or_known_default() else {
            bail!("Missing token endpoint port in {endpoint}");
        };

        match endpoint.scheme() {
            "https" => StreamStd::connect_tls(host, port, self.tls),
            "http" => StreamStd::connect_tcp(host, port),
            scheme => bail!("Unsupported token endpoint scheme: {scheme}"),
        }
    }

    fn request_builder(&self) -> Result<HttpRequest> {
        let endpoint = self.token_endpoint;

        let Some(host) = endpoint.host_str() else {
            bail!("Missing token endpoint host name in {endpoint}");
        };

        let Some(port) = endpoint.port_or_known_default() else {
            bail!("Missing token endpoint port in {endpoint}");
        };

        let mut request = HttpRequest {
            method: "POST".into(),
            url: endpoint.clone(),
            headers: Vec::new(),
            body: Vec::new(),
        }
        .header("Host", format!("{host}:{port}"));

        if let Some(secret) = &self.client_secret {
            let creds = format!("{}:{}", self.client_id, secret.expose_secret());
            let digest = BASE64_URL_SAFE_NO_PAD.encode(creds);
            request = request.header("Authorization", format!("Basic {digest}"));
        }

        Ok(request)
    }
}
