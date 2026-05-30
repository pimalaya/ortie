//! Std-blocking OAuth 2.0 client wrapping a single boxed stream.
//!
//! Two construction paths:
//! - [`OauthClientStd::new`] wraps any pre-connected
//!   `Read + Write + Send` stream. Callers own connection setup
//!   (TCP, TLS, etc.).
//! - [`OauthClientStd::connect`] (TLS-gated) opens the TCP/TLS stream
//!   itself via [`pimalaya_stream::std::stream::StreamStd`].
//!
//! Per-operation methods inline the coroutine loop against the
//! client's stream.

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener},
};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use io_http::rfc9110::request::HttpRequest;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::{std::stream::StreamStd, tls::Tls};
use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;

use crate::{
    authorization_code_grant::access_token_request::{
        AccessTokenRequestParams, RequestOauth2AccessToken, RequestOauth2AccessTokenError,
        RequestOauth2AccessTokenResult,
    },
    issue_access_token::AccessTokenResponse,
    refresh_access_token::{
        RefreshAccessTokenParams, RefreshOauth2AccessToken, RefreshOauth2AccessTokenError,
        RefreshOauth2AccessTokenResult,
    },
};

const READ_BUFFER_SIZE: usize = 8 * 1024;

/// Errors returned by [`OauthClientStd`].
#[derive(Debug, Error)]
pub enum OauthClientStdError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    RequestAccessToken(#[from] RequestOauth2AccessTokenError),
    #[error(transparent)]
    RefreshAccessToken(#[from] RefreshOauth2AccessTokenError),

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(transparent)]
    Tls(#[from] anyhow::Error),

    #[error("OAuth 2.0 URL `{0}` has no host")]
    UrlMissingHost(String),
    #[error("OAuth 2.0 URL `{0}` has no port")]
    UrlMissingPort(String),
    #[error("OAuth 2.0 URL `{0}` has unsupported scheme `{1}` (expected `http` or `https`)")]
    UrlUnsupportedScheme(String, String),

    #[error("Malformed HTTP request received on redirect server: `{0}`")]
    InvalidRedirectRequest(String),
}

/// Marker trait for streams the client wraps; implemented for any
/// blocking `Read + Write + Send` stream.
pub trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send + ?Sized> Stream for T {}

/// Std-blocking OAuth 2.0 client wrapping a single boxed stream.
pub struct OauthClientStd {
    pub stream: Box<dyn Stream>,
    pub token_endpoint: Url,
    pub client_id: String,
    pub client_secret: Option<SecretString>,
}

impl OauthClientStd {
    /// Builds a client around `stream`. The caller is responsible for
    /// opening the connection (TCP, TLS handshake if needed).
    pub fn new<S: Read + Write + Send + 'static>(
        stream: S,
        token_endpoint: Url,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            stream: Box::new(stream),
            token_endpoint,
            client_id: client_id.into(),
            client_secret: None,
        }
    }

    /// Opens a TLS-aware connection to `token_endpoint` and returns
    /// a client ready to issue requests against it.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub fn connect(
        token_endpoint: Url,
        tls: &Tls,
        client_id: impl Into<String>,
    ) -> Result<Self, OauthClientStdError> {
        let host = token_endpoint
            .host_str()
            .ok_or_else(|| OauthClientStdError::UrlMissingHost(token_endpoint.to_string()))?;
        let port = token_endpoint
            .port_or_known_default()
            .ok_or_else(|| OauthClientStdError::UrlMissingPort(token_endpoint.to_string()))?;

        let stream = match token_endpoint.scheme() {
            scheme if scheme.eq_ignore_ascii_case("https") => {
                StreamStd::connect_tls(host, port, tls)?
            }
            scheme if scheme.eq_ignore_ascii_case("http") => StreamStd::connect_tcp(host, port)?,
            scheme => {
                return Err(OauthClientStdError::UrlUnsupportedScheme(
                    token_endpoint.to_string(),
                    scheme.to_string(),
                ));
            }
        };

        Ok(Self::new(stream, token_endpoint, client_id))
    }

    /// Replaces the underlying stream.
    pub fn set_stream<S: Read + Write + Send + 'static>(&mut self, stream: S) {
        self.stream = Box::new(stream);
    }

    /// Exchanges an authorization code for an access token.
    ///
    /// `client_id` is taken from the client; the caller supplies the
    /// code-side parameters only.
    pub fn request_access_token(
        &mut self,
        params: AccessTokenRequestParams<'_>,
    ) -> Result<AccessTokenResponse, OauthClientStdError> {
        let request = self.build_post_request();
        let mut coroutine = RequestOauth2AccessToken::new(request, params);
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match coroutine.resume(arg.take()) {
                RequestOauth2AccessTokenResult::Ok(res) => return Ok(res),
                RequestOauth2AccessTokenResult::WantsRead => {
                    let n = self.stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                RequestOauth2AccessTokenResult::WantsWrite(bytes) => {
                    self.stream.write_all(&bytes)?;
                }
                RequestOauth2AccessTokenResult::Err(err) => return Err(err.into()),
            }
        }
    }

    /// Refreshes an access token using a refresh token.
    pub fn refresh_access_token(
        &mut self,
        params: RefreshAccessTokenParams<'_>,
    ) -> Result<AccessTokenResponse, OauthClientStdError> {
        let request = self.build_post_request();
        let mut coroutine = RefreshOauth2AccessToken::new(request, params);
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match coroutine.resume(arg.take()) {
                RefreshOauth2AccessTokenResult::Ok(res) => return Ok(res),
                RefreshOauth2AccessTokenResult::WantsRead => {
                    let n = self.stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                RefreshOauth2AccessTokenResult::WantsWrite(bytes) => {
                    self.stream.write_all(&bytes)?;
                }
                RefreshOauth2AccessTokenResult::Err(err) => return Err(err.into()),
            }
        }
    }

    fn build_post_request(&self) -> HttpRequest {
        let host = self.token_endpoint.host_str().unwrap_or("");
        let port = self.token_endpoint.port_or_known_default().unwrap_or(0);

        let mut request = HttpRequest {
            method: "POST".into(),
            url: self.token_endpoint.clone(),
            headers: Vec::new(),
            body: Vec::new(),
        }
        .header("Host", format!("{host}:{port}"));

        if let Some(secret) = &self.client_secret {
            let creds = format!("{}:{}", self.client_id, secret.expose_secret());
            let digest = BASE64_URL_SAFE_NO_PAD.encode(creds);
            request = request.header("Authorization", format!("Basic {digest}"));
        }

        request
    }
}

/// Binds a local TCP listener on `redirect_uri`, waits for the
/// authorization server's redirect, sends a `200 OK` placeholder
/// response, and returns the redirected URL (carrying `code` and
/// `state`).
///
/// Single-shot: the listener closes once one redirect is handled.
/// PKCE verifier and original state are tracked by the caller (this
/// fn only forwards the URL).
pub fn await_redirect(redirect_uri: &Url) -> Result<Url, OauthClientStdError> {
    let scheme = redirect_uri.scheme();
    let host = redirect_uri
        .host_str()
        .ok_or_else(|| OauthClientStdError::UrlMissingHost(redirect_uri.to_string()))?;
    let port = redirect_uri
        .port_or_known_default()
        .ok_or_else(|| OauthClientStdError::UrlMissingPort(redirect_uri.to_string()))?;

    let listener = TcpListener::bind((host, port))?;
    let (mut stream, _) = listener.accept()?;
    let mut reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let redirected_path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OauthClientStdError::InvalidRedirectRequest(request_line.clone()))?;

    let redirected_uri: Url = format!("{scheme}://{host}:{port}{redirected_path}")
        .parse()
        .map_err(|_| OauthClientStdError::InvalidRedirectRequest(request_line.clone()))?;

    let stream = reader.into_inner();
    stream.write_all(b"HTTP/1.0 200 OK\r\n\r\nAuthorization succeeded!")?;
    stream.shutdown(Shutdown::Both)?;

    Ok(redirected_uri)
}
