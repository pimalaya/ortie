//! Module dedicated to the section 5: Issuing an Access Token.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-5>

use std::time::SystemTime;

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};

pub type AccessTokenResponse = Result<IssueAccessTokenSuccessParams, IssueAccessTokenErrorParams>;

/// The response returned by the authorization server when the access
/// token request is valid and authorized.
///
/// The authorization server issues an access token and optional
/// refresh token, and constructs the response by adding the following
/// parameters to the entity-body of the HTTP response with a 200 (OK)
/// status code.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-5.1>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueAccessTokenSuccessParams {
    /// The access token issued by the authorization server.
    #[serde(serialize_with = "serialize_secret_string")]
    pub access_token: SecretString,

    /// The type of the token issued.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-7.1>
    pub token_type: String,

    /// The lifetime in seconds of the access token.
    ///
    /// For example, the value "3600" denotes that the access token
    /// will expire in one hour from the time the response was
    /// generated. If omitted, the authorization server SHOULD provide
    /// the expiration time via other means or document the default
    /// value.
    pub expires_in: Option<usize>,

    /// The refresh token.
    ///
    /// The refresh token, which can be used to obtain new access
    /// tokens using the same authorization grant.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-6>
    #[serde(serialize_with = "serialize_opt_secret_string")]
    pub refresh_token: Option<SecretString>,

    /// The scope of the access token.
    ///
    /// OPTIONAL, if identical to the scope requested by the client;
    /// otherwise, REQUIRED.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-3.3>
    pub scope: Option<String>,

    /// Time the access token was issued at.
    ///
    /// This field does not belong to the specs, its sole purpose is
    /// to track whenever the token is expired or not.
    #[serde(default = "SystemTime::now")]
    pub issued_at: SystemTime,
}

impl IssueAccessTokenSuccessParams {
    pub fn sync_expires_in(&mut self) {
        let Some(expires_in) = &mut self.expires_in else {
            return;
        };

        let Ok(elapsed) = self.issued_at.elapsed() else {
            return;
        };

        let elapsed = elapsed.as_secs() as usize;

        *expires_in -= elapsed.min(*expires_in);
    }
}

/// Serializes success params into JSON string.
// SAFETY: exposes access and refresh tokens
impl TryFrom<&IssueAccessTokenSuccessParams> for String {
    type Error = serde_json::Error;

    fn try_from(params: &IssueAccessTokenSuccessParams) -> Result<Self, Self::Error> {
        serde_json::to_string(params)
    }
}

/// Deserializes success params from JSON bytes.
impl TryFrom<&[u8]> for IssueAccessTokenSuccessParams {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

/// The response returned by the authorization server when the access
/// token request is not valid or unauthorized.
///
/// The authorization server responds with an HTTP 400 (Bad Request)
/// status code (unless specified otherwise) and includes the
/// following parameters with the response.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-5.2>
#[derive(Clone, Debug, Deserialize)]
pub struct IssueAccessTokenErrorParams {
    /// A single ASCII error code.
    pub error: IssueAccessTokenErrorCode,

    /// Human-readable ASCII text providing additional information,
    /// used to assist the client developer in understanding the error
    /// that occurred.
    pub error_description: Option<String>,

    /// A URI identifying a human-readable web page with information
    /// about the error, used to provide the client developer with
    /// additional information about the error.
    pub error_uri: Option<String>,
}

/// Parses error params for JSON bytes.
impl TryFrom<&[u8]> for IssueAccessTokenErrorParams {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

/// The error code of the [`IssueAccessTokenErrorParams`].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueAccessTokenErrorCode {
    /// Client authentication failed (e.g., unknown client, no client
    /// authentication included, or unsupported authentication
    /// method).  The authorization server MAY return an HTTP 401
    /// (Unauthorized) status code to indicate which HTTP
    /// authentication schemes are supported.  If the client attempted
    /// to authenticate via the "Authorization" request header field,
    /// the authorization server MUST respond with an HTTP 401
    /// (Unauthorized) status code and include the "WWW-Authenticate"
    /// response header field matching the authentication scheme used
    /// by the client.
    InvalidClient,

    /// The provided authorization grant (e.g., authorization code,
    /// resource owner credentials) or refresh token is invalid,
    /// expired, revoked, does not match the redirection URI used in
    /// the authorization request, or was issued to another client.
    InvalidGrant,

    /// The request is missing a required parameter, includes an
    /// unsupported parameter value (other than grant type), repeats a
    /// parameter, includes multiple credentials, utilizes more than
    /// one mechanism for authenticating the client, or is otherwise
    /// malformed.
    InvalidRequest,

    /// The requested scope is invalid, unknown, malformed, or exceeds
    /// the scope granted by the resource owner.
    InvalidScope,

    /// The authenticated client is not authorized to use this
    /// authorization grant type.
    UnauthorizedClient,

    /// The authorization grant type is not supported by the
    /// authorization server.
    UnsupportedGrantType,
}

fn serialize_secret_string<S: Serializer>(secret: &SecretString, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(secret.expose_secret())
}

fn serialize_opt_secret_string<S: Serializer>(
    secret: &Option<SecretString>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match secret {
        Some(secret) => serialize_secret_string(secret, s),
        None => s.serialize_none(),
    }
}
