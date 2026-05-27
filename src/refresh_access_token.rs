//! Module dedicated to the section 6: Refreshing an Access Token.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-6>

use std::{borrow::Cow, collections::HashSet};

use io_http::{
    rfc9110::request::HttpRequest,
    rfc9112::send::{Http11Send, Http11SendError, Http11SendResult},
};
use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::{form_urlencoded::Serializer, Url};

use crate::issue_access_token::{
    AccessTokenResponse, IssueAccessTokenErrorParams, IssueAccessTokenSuccessParams,
};

/// Errors that can occur during the coroutine progression.
#[derive(Debug, Error)]
pub enum RefreshOauth2AccessTokenError {
    #[error(transparent)]
    SendHttpRefresh(#[from] Http11SendError),
    #[error(transparent)]
    ParseHttpResponse(#[from] serde_json::Error),
    #[error("Unexpected redirection {code} to {url}")]
    Redirect { url: Url, code: u16 },
    #[error("HTTP error: status {0}")]
    Status(u16),
}

/// Result returned by the coroutine's resume function.
#[derive(Debug)]
pub enum RefreshOauth2AccessTokenResult {
    /// The coroutine has successfully terminated its execution.
    Ok(AccessTokenResponse),
    /// The coroutine wants the socket to be read into.
    WantsRead,
    /// The coroutine wants the given bytes to be written to the
    /// socket.
    WantsWrite(Vec<u8>),
    /// The coroutine encountered an error.
    Err(RefreshOauth2AccessTokenError),
}

/// The I/O-free coroutine to refresh an access token.
///
/// This coroutine sends the refresh access token HTTP request to the
/// token endpoint and receives either a successful or an error HTTP
/// response.
///
/// Refs: [`AccessTokenResponse`]
pub struct RefreshOauth2AccessToken {
    send: Http11Send,
}

impl RefreshOauth2AccessToken {
    /// Creates a new I/O-free coroutine to refresh an access token.
    pub fn new(request: HttpRequest, body: RefreshAccessTokenParams<'_>) -> Self {
        let request = request
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string().into_bytes());

        Self {
            send: Http11Send::new(request),
        }
    }

    /// Makes the coroutine progress.
    pub fn resume(&mut self, arg: Option<&[u8]>) -> RefreshOauth2AccessTokenResult {
        match self.send.resume(arg) {
            Http11SendResult::Ok { response, .. } if response.status.is_success() => {
                match IssueAccessTokenSuccessParams::try_from(response.body.as_slice()) {
                    Ok(res) => RefreshOauth2AccessTokenResult::Ok(Ok(res)),
                    Err(err) => RefreshOauth2AccessTokenResult::Err(err.into()),
                }
            }
            Http11SendResult::Ok { response, .. } => {
                match IssueAccessTokenErrorParams::try_from(response.body.as_slice()) {
                    Ok(res) => RefreshOauth2AccessTokenResult::Ok(Err(res)),
                    Err(err) => RefreshOauth2AccessTokenResult::Err(err.into()),
                }
            }
            Http11SendResult::WantsRead => RefreshOauth2AccessTokenResult::WantsRead,
            Http11SendResult::WantsWrite(bytes) => {
                RefreshOauth2AccessTokenResult::WantsWrite(bytes)
            }
            Http11SendResult::WantsRedirect { url, response, .. } => {
                RefreshOauth2AccessTokenResult::Err(RefreshOauth2AccessTokenError::Redirect {
                    url,
                    code: *response.status,
                })
            }
            Http11SendResult::Err(err) => RefreshOauth2AccessTokenResult::Err(err.into()),
        }
    }
}

/// The refresh access token request parameters.
///
/// If the authorization server issued a refresh token to the client,
/// the client makes a refresh request to the token endpoint by adding
/// the following parameters using the
/// "application/x-www-form-urlencoded" format with a character
/// encoding of UTF-8 in the HTTP request entity-body.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-6>
#[derive(Debug)]
pub struct RefreshAccessTokenParams<'a> {
    pub client_id: String,
    pub refresh_token: SecretString,
    pub scopes: HashSet<Cow<'a, str>>,
}

impl<'a> RefreshAccessTokenParams<'a> {
    pub fn new(client_id: impl ToString, refresh_token: impl Into<SecretString>) -> Self {
        Self {
            client_id: client_id.to_string(),
            refresh_token: refresh_token.into(),
            scopes: HashSet::new(),
        }
    }

    pub fn to_serializer(&self) -> Serializer<'a, String> {
        let mut serializer = Serializer::new(String::new());

        serializer.append_pair("grant_type", "refresh_token");
        serializer.append_pair("client_id", &self.client_id);
        serializer.append_pair("refresh_token", self.refresh_token.expose_secret());

        if !self.scopes.is_empty() {
            let mut scope = String::new();
            let mut glue = "";

            for token in &self.scopes {
                scope.push_str(glue);
                scope.push_str(token);
                glue = " ";
            }

            serializer.append_pair("scope", &scope);
        }

        serializer
    }
}

impl ToString for RefreshAccessTokenParams<'_> {
    fn to_string(&self) -> String {
        self.to_serializer().finish()
    }
}
