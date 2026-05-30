//! Module dedicated to the section 5: Issuing an Access Token.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-5>

use alloc::string::String;

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

    /// Unix epoch seconds when the access token was issued.
    ///
    /// Outside the OAuth 2.0 specs; populated by the coroutine from
    /// the HTTP `Date` response header (RFC 9110 §5.6.7). `None`
    /// when the server did not advertise a `Date`. Callers compute
    /// expiration as `issued_at + expires_in` against their own
    /// clock.
    #[serde(default)]
    pub issued_at: Option<u64>,
}

/// Parse an HTTP IMF-fixdate string (RFC 9110 §5.6.7) into Unix
/// epoch seconds (UTC).
///
/// Format: `Sun, 06 Nov 1994 08:49:37 GMT` (29 ASCII bytes). Returns
/// `None` on any structural deviation. Does not validate that the
/// day-of-month is legal for the month/year; relies on origin
/// servers to send well-formed dates.
pub fn parse_http_date(s: &str) -> Option<u64> {
    let b = s.as_bytes();

    if b.len() != 29 || &b[26..29] != b"GMT" {
        return None;
    }

    let day = parse_2_digits(&b[5..7])? as u64;
    let month: u64 = match &b[8..11] {
        b"Jan" => 1,
        b"Feb" => 2,
        b"Mar" => 3,
        b"Apr" => 4,
        b"May" => 5,
        b"Jun" => 6,
        b"Jul" => 7,
        b"Aug" => 8,
        b"Sep" => 9,
        b"Oct" => 10,
        b"Nov" => 11,
        b"Dec" => 12,
        _ => return None,
    };
    let year = parse_4_digits(&b[12..16])? as u64;
    let hour = parse_2_digits(&b[17..19])? as u64;
    let min = parse_2_digits(&b[20..22])? as u64;
    let sec = parse_2_digits(&b[23..25])? as u64;

    // NOTE: Howard Hinnant's days_from_civil algorithm; treats March
    // as the first month so the leap day lands at the end of the year.
    let (y, m) = if month <= 2 {
        (year - 1, month + 9)
    } else {
        (year, month - 3)
    };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days_from_epoch = era * 146097 + doe - 719468;

    Some(days_from_epoch * 86400 + hour * 3600 + min * 60 + sec)
}

fn parse_2_digits(b: &[u8]) -> Option<u32> {
    let a = (b[0] as u32).wrapping_sub(b'0' as u32);
    let c = (b[1] as u32).wrapping_sub(b'0' as u32);
    if a > 9 || c > 9 {
        return None;
    }
    Some(a * 10 + c)
}

fn parse_4_digits(b: &[u8]) -> Option<u32> {
    Some(parse_2_digits(&b[0..2])? * 100 + parse_2_digits(&b[2..4])?)
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
