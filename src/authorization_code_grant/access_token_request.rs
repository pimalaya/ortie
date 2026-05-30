//! Module dedicated to the section 4.1.3: Access Token Request.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3>

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use io_http::{
    coroutine::*,
    rfc9110::{
        request::HttpRequest,
        send::{HttpSendOutput, HttpSendYield},
    },
    rfc9112::send::{Http11Send, Http11SendError},
};
use thiserror::Error;
use url::{Url, form_urlencoded::Serializer};

use crate::issue_access_token::{
    AccessTokenResponse, IssueAccessTokenErrorParams, IssueAccessTokenSuccessParams,
    parse_http_date,
};

#[cfg(feature = "pkce")]
use crate::authorization_code_grant::pkce::PkceCodeVerifier;

pub struct AccessTokenRequestParams<'a> {
    pub code: Cow<'a, str>,
    pub redirect_uri: Option<Cow<'a, str>>,
    pub client_id: Cow<'a, str>,
    #[cfg(feature = "pkce")]
    pub pkce_code_verifier: Option<Cow<'a, PkceCodeVerifier>>,
}

impl<'a> AccessTokenRequestParams<'a> {
    // SAFETY: this function exposes the code and the PKCE code
    // verifier
    pub fn to_form_url_encoded_serializer(&self) -> Serializer<'a, String> {
        let mut serializer = Serializer::new(String::new());

        serializer.append_pair("grant_type", "authorization_code");
        serializer.append_pair("code", &self.code);

        if let Some(uri) = &self.redirect_uri {
            serializer.append_pair("redirect_uri", uri);
        }

        serializer.append_pair("client_id", &self.client_id);

        #[cfg(feature = "pkce")]
        if let Some(verifier) = &self.pkce_code_verifier {
            let verifier = String::from_utf8_lossy(verifier.expose());
            serializer.append_pair("code_verifier", &verifier);
        }

        serializer
    }
}

impl fmt::Display for AccessTokenRequestParams<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_form_url_encoded_serializer().finish())
    }
}

/// Errors that can occur during the coroutine progression.
#[derive(Debug, Error)]
pub enum RequestOauth2AccessTokenError {
    #[error(transparent)]
    SendHttpRequest(#[from] Http11SendError),
    #[error(transparent)]
    ParseHttpResponse(#[from] serde_json::Error),
    #[error("Unexpected redirection {code} to {url}")]
    Redirect { url: Url, code: u16 },
}

/// Result returned by the coroutine's resume function.
#[derive(Debug)]
pub enum RequestOauth2AccessTokenResult {
    /// The coroutine has successfully terminated its execution.
    Ok(AccessTokenResponse),
    /// The coroutine wants the socket to be read into.
    WantsRead,
    /// The coroutine wants the given bytes to be written to the
    /// socket.
    WantsWrite(Vec<u8>),
    /// The coroutine encountered an error.
    Err(RequestOauth2AccessTokenError),
}

/// The authorization code grant type is used to obtain both access
/// tokens and refresh tokens and is optimized for confidential
/// clients. Since this is a redirection-based flow, the client must
/// be capable of interacting with the resource owner's user-agent
/// (typically a web browser) and capable of receiving incoming
/// requests (via redirection) from the authorization server.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1>
#[derive(Debug)]
pub struct RequestOauth2AccessToken {
    send: Http11Send,
}

impl RequestOauth2AccessToken {
    pub fn new(request: HttpRequest, body: AccessTokenRequestParams<'_>) -> Self {
        let request = request
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string().into_bytes());

        Self {
            send: Http11Send::new(request),
        }
    }

    pub fn resume(&mut self, arg: Option<&[u8]>) -> RequestOauth2AccessTokenResult {
        match self.send.resume(arg) {
            HttpCoroutineState::Complete(Ok(HttpSendOutput { response, .. }))
                if response.status.is_success() =>
            {
                match IssueAccessTokenSuccessParams::try_from(response.body.as_slice()) {
                    Ok(mut res) => {
                        res.issued_at = response.header("date").and_then(parse_http_date);
                        RequestOauth2AccessTokenResult::Ok(Ok(res))
                    }
                    Err(err) => RequestOauth2AccessTokenResult::Err(err.into()),
                }
            }
            HttpCoroutineState::Complete(Ok(HttpSendOutput { response, .. })) => {
                match IssueAccessTokenErrorParams::try_from(response.body.as_slice()) {
                    Ok(res) => RequestOauth2AccessTokenResult::Ok(Err(res)),
                    Err(err) => RequestOauth2AccessTokenResult::Err(err.into()),
                }
            }
            HttpCoroutineState::Yielded(HttpSendYield::WantsRead) => {
                RequestOauth2AccessTokenResult::WantsRead
            }
            HttpCoroutineState::Yielded(HttpSendYield::WantsWrite(bytes)) => {
                RequestOauth2AccessTokenResult::WantsWrite(bytes)
            }
            HttpCoroutineState::Yielded(HttpSendYield::WantsRedirect { url, response, .. }) => {
                RequestOauth2AccessTokenResult::Err(RequestOauth2AccessTokenError::Redirect {
                    url,
                    code: *response.status,
                })
            }
            HttpCoroutineState::Complete(Err(err)) => {
                RequestOauth2AccessTokenResult::Err(err.into())
            }
        }
    }
}
