//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected
//! to act on, printed as `code` beside the message under `--json`. The
//! message stays for humans and may be reworded at any time; the code is
//! the contract. A failure with no code prints `"code": null`. The
//! mechanism (`CodedError`, `code_of`, `with_code`, the report) lives in
//! `pimalaya_cli::error`; only the vocabulary and [`code`] are Ortie's.
//!
//! The shared vocabulary names a refusal by the HTTP status alone:
//! `permission-denied` for a 403 and `rate-limited` for a 429, read from
//! io-oauth's typed client error. Ortie's own codes name the OAuth
//! refusals, each by what the caller does next: run `auth get` again,
//! fix the client configuration, or nothing. The account codes are read
//! from pimalaya-config's `AccountError`.
//!
//! io-oauth's OAuth error bodies are no `std::error::Error`, so they are
//! coded where Ortie turns them into a failure ([`oauth_error`]), its
//! message naming the server's code as the server spelled it.

use std::{error::Error as StdError, fmt::Display};

use anyhow::Error;
use io_oauth::{
    client::Oauth20ClientStdError,
    rfc6749::{
        access_token_request::Oauth20AccessTokenRequestError, auth_response::Oauth20AuthErrorCode,
        client_credentials::Oauth20ClientCredentialsRequestError,
        issue_access_token::Oauth20AccessTokenErrorCode,
        refresh_access_token::Oauth20AccessTokenRefreshError,
    },
    rfc7009::revoke::{Oauth20TokenRevokeError, Oauth20TokenRevokeErrorCode},
    rfc7523::{
        auth_grant::Oauth20JwtBearerGrantRequestError,
        client_auth::Oauth20JwtBearerClientCredentialsRequestError,
    },
    rfc7591::register::Oauth20ClientRegisterError,
    rfc8628::{auth::Oauth20DeviceAuthRequestError, token::Oauth20DeviceAccessTokenRequestError},
};
use pimalaya_cli::error::{CodedError, CommonCode, code_of};
use pimalaya_config::toml::AccountError;
use schemars::JsonSchema;
use serde::Serialize;

/// The stable codes, each serialised in kebab-case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The grant was refused to the user or the client: `access_denied` at
    /// the authorization redirect or in the device flow, Microsoft Entra's
    /// `authorization_declined`, or a bare HTTP 403.
    PermissionDenied,
    /// The server answered a bare HTTP 429; Ortie retries nothing.
    RateLimited,
    /// `invalid_grant`: the authorization code, the refresh token or the
    /// device code was refused, has expired or was revoked; run `auth
    /// get` again.
    GrantInvalid,
    /// `invalid_client`: the client secret, certificate or assertion was
    /// refused.
    ClientInvalid,
    /// `unauthorized_client`: the client may not use the grant.
    ClientUnauthorized,
    /// `invalid_scope`: a requested scope is invalid, unknown or exceeds
    /// the granted one.
    ScopeInvalid,
    /// The device code expired before the user approved it, by the
    /// server's `expired_token` or by the local deadline.
    DeviceCodeExpired,
    /// The authorization redirect carries no `state`, or another one than
    /// the request sent.
    StateMismatch,
    /// The account stores no token, or no refresh token where a refresh
    /// needs one; run `auth get`.
    TokenMissing,
    /// The stored token expired and holds no refresh token, on a grant
    /// that cannot run again by itself; run `auth get`.
    TokenExpired,
    /// The account configures no endpoint the grant needs: token, device
    /// authorization, authorization or revocation.
    EndpointMissing,
    /// The account's `endpoints.redirection` is not an http URI on a
    /// loopback host, which `auth listen` cannot bind; `auth resume` with
    /// the redirected URI finishes the flow.
    RedirectionUnsupported,
    /// The user did not agree to the Google data disclosure of Pimalaya's
    /// application, at the prompt or by `--agree-data-use`.
    AgreementMissing,
    /// No account has the name `-a` gave.
    AccountMissing,
    /// No account is marked default, and `-a` named none.
    AccountDefaultMissing,
    /// Several accounts are marked default, and `-a` named none.
    AccountDefaultSeveral,
    /// A command asked for its input on stdin inside a session, which
    /// reads its own lines there.
    StdinTaken,
}

/// An OAuth error code of a server's answer: its wire spelling and the
/// code it carries.
pub trait OauthErrorCode {
    /// The code as the server spelled it, `None` for one io-oauth does not
    /// register, whose word it drops.
    fn wire(&self) -> Option<&'static str>;

    /// The code the failure carries, `None` for one no caller acts on.
    fn code(&self) -> Option<ErrorCode>;
}

impl OauthErrorCode for Oauth20AccessTokenErrorCode {
    fn wire(&self) -> Option<&'static str> {
        Some(match self {
            Self::InvalidClient => "invalid_client",
            Self::InvalidGrant => "invalid_grant",
            Self::InvalidRequest => "invalid_request",
            Self::InvalidScope => "invalid_scope",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::AuthorizationPending => "authorization_pending",
            Self::SlowDown => "slow_down",
            Self::AccessDenied => "access_denied",
            Self::ExpiredToken => "expired_token",
            Self::AuthorizationDeclined => "authorization_declined",
            Self::BadVerificationCode => "bad_verification_code",
            Self::InvalidTarget => "invalid_target",
            Self::Unknown => return None,
        })
    }

    fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::InvalidClient => Some(ErrorCode::ClientInvalid),
            Self::InvalidGrant => Some(ErrorCode::GrantInvalid),
            Self::InvalidScope => Some(ErrorCode::ScopeInvalid),
            Self::UnauthorizedClient => Some(ErrorCode::ClientUnauthorized),
            Self::AccessDenied | Self::AuthorizationDeclined => Some(ErrorCode::PermissionDenied),
            Self::ExpiredToken => Some(ErrorCode::DeviceCodeExpired),
            _ => None,
        }
    }
}

impl OauthErrorCode for Oauth20AuthErrorCode {
    fn wire(&self) -> Option<&'static str> {
        Some(match self {
            Self::InvalidRequest => "invalid_request",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::AccessDenied => "access_denied",
            Self::UnsupportedResponseType => "unsupported_response_type",
            Self::InvalidScope => "invalid_scope",
            Self::ServerError => "server_error",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
            Self::InvalidTarget => "invalid_target",
            Self::Unknown => return None,
        })
    }

    fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::UnauthorizedClient => Some(ErrorCode::ClientUnauthorized),
            Self::AccessDenied => Some(ErrorCode::PermissionDenied),
            Self::InvalidScope => Some(ErrorCode::ScopeInvalid),
            _ => None,
        }
    }
}

impl OauthErrorCode for Oauth20TokenRevokeErrorCode {
    fn wire(&self) -> Option<&'static str> {
        Some(match self {
            Self::InvalidClient => "invalid_client",
            Self::InvalidRequest => "invalid_request",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::UnsupportedTokenType => "unsupported_token_type",
            Self::Unknown => return None,
        })
    }

    fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::InvalidClient => Some(ErrorCode::ClientInvalid),
            Self::UnauthorizedClient => Some(ErrorCode::ClientUnauthorized),
            _ => None,
        }
    }
}

/// The failure an OAuth error answer reads as: `action` naming the
/// server's code, caused by its description and URI, and the `hint`
/// Ortie adds, coded when the vocabulary has one.
pub fn oauth_error(
    action: &str,
    error: &impl OauthErrorCode,
    description: Option<impl Display>,
    uri: Option<impl Display>,
    hint: Option<&str>,
) -> Error {
    let top = match error.wire() {
        Some(wire) => format!("{action} (code {wire})"),
        None => format!("{action} (unregistered code)"),
    };

    let detail = match (description, uri) {
        (None, None) => None,
        (Some(desc), None) => Some(desc.to_string()),
        (None, Some(uri)) => Some(uri.to_string()),
        (Some(desc), Some(uri)) => Some(format!("{desc}: {uri}")),
    };

    // NOTE: the innermost message carries the code, the others wrap it.
    let mut messages: Vec<_> = [Some(top), detail, hint.map(String::from)]
        .into_iter()
        .flatten()
        .collect();
    let innermost = messages.pop().unwrap_or_default();
    let err = match error.code() {
        Some(code) => coded(code, innermost),
        None => Error::msg(innermost),
    };

    messages
        .into_iter()
        .rev()
        .fold(err, |err, message| err.context(message))
}

/// A failure carrying `code`, with its own message.
pub fn coded(code: ErrorCode, message: impl Into<String>) -> Error {
    CodedError::new(code, message).into()
}

/// A command line Ortie refuses after clap parsed it, with `message`:
/// coded `usage-invalid`, so it exits 2 as one clap refused.
pub fn usage(message: impl Into<String>) -> Error {
    CodedError::new(CommonCode::UsageInvalid, message).into()
}

/// The failure of an account configuring no `key` endpoint, named as
/// the configuration writes it (`endpoints.token`).
pub fn endpoint_missing(key: &str) -> Error {
    let message = format!("Missing {key} in the account config");
    coded(ErrorCode::EndpointMissing, message)
}

/// The code of `err`: the one a coded error in its chain carries, else
/// the one io-oauth's typed client error or pimalaya-config's account
/// resolution error names. pimalaya-cli's `with_code` hands it to the
/// failure object.
pub fn code(err: &Error) -> Option<ErrorCode> {
    let typed = |cause| client_code(cause).or_else(|| account_code(cause));
    code_of(err).or_else(|| err.chain().find_map(typed))
}

/// The code io-oauth's typed client error `cause` names: an expired
/// device code, or a bare 403 or 429 answered without an OAuth body.
fn client_code(cause: &(dyn StdError + 'static)) -> Option<ErrorCode> {
    use Oauth20ClientStdError as E;

    let status = match cause.downcast_ref::<Oauth20ClientStdError>()? {
        E::DeviceCodeExpired => return Some(ErrorCode::DeviceCodeExpired),
        E::AccessTokenRequest(Oauth20AccessTokenRequestError::Status(status))
        | E::AccessTokenRefresh(Oauth20AccessTokenRefreshError::Status(status))
        | E::ClientCredentialsRequest(Oauth20ClientCredentialsRequestError::Status(status))
        | E::DeviceAuthRequest(Oauth20DeviceAuthRequestError::Status(status))
        | E::DeviceAccessTokenRequest(Oauth20DeviceAccessTokenRequestError::Status(status))
        | E::JwtBearerGrantRequest(Oauth20JwtBearerGrantRequestError::Status(status))
        | E::JwtBearerClientCredentialsRequest(
            Oauth20JwtBearerClientCredentialsRequestError::Status(status),
        )
        | E::TokenRevoke(Oauth20TokenRevokeError::Status(status))
        | E::ClientRegister(Oauth20ClientRegisterError::Status(status)) => *status,
        _ => return None,
    };

    match status {
        403 => Some(ErrorCode::PermissionDenied),
        429 => Some(ErrorCode::RateLimited),
        _ => None,
    }
}

/// The code of pimalaya-config's account resolution error `cause`.
fn account_code(cause: &(dyn StdError + 'static)) -> Option<ErrorCode> {
    Some(match cause.downcast_ref::<AccountError>()? {
        AccountError::Missing { .. } => ErrorCode::AccountMissing,
        AccountError::NoDefault => ErrorCode::AccountDefaultMissing,
        AccountError::SeveralDefaults(_) => ErrorCode::AccountDefaultSeveral,
    })
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::error::with_code;
    use serde::de::DeserializeOwned;

    use super::*;

    /// Every wire spelling reads back, through io-oauth's own serde
    /// names, as the code it spells.
    fn assert_wire_round_trips<C>(codes: &[C])
    where
        C: OauthErrorCode + DeserializeOwned + std::fmt::Debug,
    {
        for code in codes {
            let wire = code.wire().unwrap();
            let read: C = serde_json::from_value(wire.into()).unwrap();
            assert_eq!(read.wire(), Some(wire), "{code:?}");
        }
    }

    #[test]
    fn the_wire_spelling_is_io_oauths_serde_name() {
        use Oauth20AccessTokenErrorCode as T;
        assert_wire_round_trips(&[
            T::InvalidClient,
            T::InvalidGrant,
            T::InvalidRequest,
            T::InvalidScope,
            T::UnauthorizedClient,
            T::UnsupportedGrantType,
            T::AuthorizationPending,
            T::SlowDown,
            T::AccessDenied,
            T::ExpiredToken,
            T::AuthorizationDeclined,
            T::BadVerificationCode,
            T::InvalidTarget,
        ]);

        use Oauth20AuthErrorCode as A;
        assert_wire_round_trips(&[
            A::InvalidRequest,
            A::UnauthorizedClient,
            A::AccessDenied,
            A::UnsupportedResponseType,
            A::InvalidScope,
            A::ServerError,
            A::TemporarilyUnavailable,
            A::InvalidTarget,
        ]);

        use Oauth20TokenRevokeErrorCode as R;
        assert_wire_round_trips(&[
            R::InvalidClient,
            R::InvalidRequest,
            R::UnauthorizedClient,
            R::UnsupportedTokenType,
        ]);
    }

    #[test]
    fn an_oauth_error_keeps_its_chain_and_carries_its_code() {
        let err = oauth_error(
            "Refresh access token error",
            &Oauth20AccessTokenErrorCode::InvalidGrant,
            Some("Token has been expired or revoked."),
            None::<String>,
            None,
        );

        assert_eq!(code(&err), Some(ErrorCode::GrantInvalid));
        assert_eq!(
            err.chain().map(ToString::to_string).collect::<Vec<_>>(),
            [
                "Refresh access token error (code invalid_grant)",
                "Token has been expired or revoked.",
            ]
        );
    }

    #[test]
    fn a_bare_status_is_coded_with_its_messages_kept() {
        let status = Oauth20AccessTokenRefreshError::Status(429);
        let err = Error::from(Oauth20ClientStdError::from(status)).context("Refresh token");
        let messages: Vec<_> = err.chain().map(ToString::to_string).collect();

        assert_eq!(code_of::<ErrorCode>(&err), None);
        assert_eq!(code(&err), Some(ErrorCode::RateLimited));

        let err = with_code(err, code);
        assert_eq!(code_of::<ErrorCode>(&err), Some(ErrorCode::RateLimited));
        assert_eq!(
            err.chain().map(ToString::to_string).collect::<Vec<_>>(),
            messages
        );
    }

    #[test]
    fn another_status_is_left_uncoded() {
        let status = Oauth20TokenRevokeError::Status(503);
        let err = Error::from(Oauth20ClientStdError::from(status));

        assert_eq!(code(&err), None);
        assert_eq!(code_of::<ErrorCode>(&with_code(err, code)), None);
    }
}
