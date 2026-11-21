//! # Auth resume command
//!
//! The `auth resume` subcommand, completing an OAuth 2.0 grant flow
//! [`crate::auth::get`] started but could not finish itself.

use std::{borrow::Cow, ffi::OsStr};

use anyhow::{Result, anyhow};
use clap::{Arg, Command, Parser, builder::TypedValueParser, error::ErrorKind};
use pimalaya_cli::printer::Printer;
use secrecy::SecretString;
use url::Url;

use pimalaya_config::secret::Secret;

use io_oauth::{
    client::Oauth20ClientStd,
    rfc6749::{
        access_token_request::Oauth20AccessTokenRequestParams,
        auth_response::{Oauth20AuthParams, Oauth20AuthParamsValidationError},
        state::Oauth20State,
    },
    rfc7636::pkce::Oauth20PkceCodeVerifier,
    rfc8628::auth::Oauth20DeviceAuthSuccessParams,
};

use crate::{
    account::Account,
    auth::get::{complete_device_token_poll, report_token_issued},
    config::GrantConfig,
    error::{self, ErrorCode},
};

/// Resume an existing OAuth 2.0 grant flow.
///
/// Completes the grant configured on the account, from the redirected
/// URI for the authorization code grant or the device code for the
/// device grant.
///
/// The authorization-code-only flags (`--state`, `--pkce`,
/// `--redirect-uri`) are rejected on a device account, and the client
/// credentials grants have nothing to resume at all.
///
/// On the authorization-code grant, --handoff reads the object `auth get
/// --json` printed, from a file or stdin, in place of --state and --pkce.
#[derive(Debug, Parser)]
pub struct AuthResumeCommand {
    /// Redirected URI, or device code on the device grant.
    ///
    /// For the authorization code grant this is the URI the browser was
    /// redirected to after consent, not the registered redirect URI.
    /// For the device grant it is the `device_code` a non-interactive
    /// or `--json` auth get returned.
    #[arg(value_name = "URI|DEVICE_CODE")]
    pub input: String,
    /// The state generated during the authorization flow initiation.
    ///
    /// Authorization-code grant only. A state generated during auth get
    /// must be given here, and must match.
    #[arg(long, short, value_parser = state_parser)]
    #[arg(value_name = "VALUE")]
    pub state: Option<Oauth20State>,
    /// The PKCE code verifier generated during the initiation.
    ///
    /// Authorization-code grant only. The verifier generated during
    /// auth get must be given here when PKCE was enabled.
    #[arg(long, short, value_parser = PkceCodeVerifierParser)]
    #[arg(value_name = "CODE")]
    pub pkce: Option<Oauth20PkceCodeVerifier>,
    /// The redirect URI used during the initiation.
    ///
    /// Authorization-code grant only. A redirect URI provided during
    /// auth get must match here.
    #[arg(long, short, value_parser = uri_parser)]
    pub redirect_uri: Option<Url>,
    /// The flow `auth get --json` printed, a path or `-` for stdin.
    ///
    /// Gives the state, the PKCE verifier and, unless --redirect-uri is
    /// given, the redirect URI, keeping them off the command line, where
    /// --state and --pkce are visible to other local users while the
    /// command runs. Authorization-code grant only.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["state", "pkce"])]
    pub handoff: Option<String>,
}

impl AuthResumeCommand {
    /// Completes the account's configured grant into a stored token.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        if account.grant.is_client_credentials() {
            let message =
                "The client credentials grants complete in a single auth get, nothing to resume";
            return Err(error::usage(message));
        }

        if account.grant == GrantConfig::Device {
            return self.execute_device(printer, account);
        }

        let Some(token_endpoint) = account.token_endpoint.clone() else {
            return Err(error::endpoint_missing("endpoints.token"));
        };

        // NOTE: the URI is never echoed back, since a pasted one still
        // carries the authorization code.
        let redirected_uri = Url::parse(self.input.trim())
            .map_err(|err| error::usage(format!("Invalid redirected URI: {err}")))?;

        // NOTE: an account names no issuer, so the `iss` of the response
        // (RFC 9207) has nothing to be compared with.
        let params = Oauth20AuthParams::from(&redirected_uri);

        let code = match params.validate(self.state.as_ref(), None) {
            Ok(code) => code,
            Err(Oauth20AuthParamsValidationError::Server(params)) => {
                return Err(error::oauth_error(
                    "Authorization error",
                    &params.error,
                    params.error_description,
                    params.error_uri,
                    None,
                ));
            }
            Err(Oauth20AuthParamsValidationError::StateMissing) => {
                let message = "Authorization response is missing state";
                return Err(error::coded(ErrorCode::StateMismatch, message));
            }
            Err(Oauth20AuthParamsValidationError::StateMismatch) => {
                let message = "Authorization request and response states do not match";
                return Err(error::coded(ErrorCode::StateMismatch, message));
            }
            Err(err @ Oauth20AuthParamsValidationError::IssuerMismatch(_)) => {
                return Err(anyhow!("{err}"));
            }
        };

        let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;

        let redirect_uri = self
            .redirect_uri
            .as_ref()
            .map(|uri| Cow::Owned(uri.to_string()))
            .or_else(|| {
                account
                    .redirection_endpoint
                    .as_ref()
                    .map(|uri| Cow::Owned(uri.to_string()))
            });

        let mut client =
            Oauth20ClientStd::connect(token_endpoint, &account.tls, account.client_id.clone())?;
        client.client_secret = client_secret;

        let res = client.request_access_token(Oauth20AccessTokenRequestParams {
            code,
            redirect_uri,
            client_id: account.client_id.as_str().into(),
            client_secret: None,
            pkce_code_verifier: self.pkce.as_ref().map(Cow::Borrowed),
            resource: account.resources.clone(),
        })?;

        match res {
            Ok(res) => printer.out(report_token_issued(account, &res)?),
            Err(res) => Err(error::oauth_error(
                "Issue access token error",
                &res.error,
                res.error_description,
                res.error_uri,
                None,
            )),
        }
    }

    /// Takes `input` as the device code and polls the token endpoint.
    fn execute_device(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        if self.state.is_some() || self.pkce.is_some() || self.redirect_uri.is_some() {
            let message = "The --state, --pkce and --redirect-uri flags are only valid \
                           for the authorization-code grant";
            return Err(error::usage(message));
        }

        let device_code = self.input.trim();
        if device_code.is_empty() {
            return Err(error::usage("Missing device code"));
        }

        let Some(token_endpoint) = account.token_endpoint.clone() else {
            return Err(error::endpoint_missing("endpoints.token"));
        };

        // NOTE: a resume is handed the device code alone, so the poll
        // loop falls back on the RFC 8628 example values.
        let device = Oauth20DeviceAuthSuccessParams {
            device_code: SecretString::from(device_code),
            user_code: String::new(),
            verification_uri: String::new(),
            verification_uri_complete: None,
            expires_in: 1800,
            interval: 5,
        };

        printer.out(complete_device_token_poll(
            account,
            &token_endpoint,
            &device,
        )?)
    }
}

/// Clap value parser for URI arguments.
pub fn uri_parser(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|err| err.to_string())
}

/// Clap value parser for the CSRF state argument.
pub fn state_parser(state: &str) -> Result<Oauth20State, String> {
    Oauth20State::try_from(state.as_bytes()).map_err(|err| err.to_string())
}

/// Clap value parser for the PKCE code verifier argument.
///
/// Its refusal names a length or a byte and the flag, never the value,
/// which clap's own refusal of a parser's error would echo.
#[derive(Clone, Copy, Debug)]
pub struct PkceCodeVerifierParser;

impl TypedValueParser for PkceCodeVerifierParser {
    type Value = Oauth20PkceCodeVerifier;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        Oauth20PkceCodeVerifier::try_from(value.as_encoded_bytes()).map_err(|err| {
            let arg = arg.map(ToString::to_string).unwrap_or_default();
            let message = format!("invalid value for '{arg}': {err}\n");
            clap::Error::raw(ErrorKind::ValueValidation, message).with_cmd(cmd)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use clap::{Command, builder::TypedValueParser};

    use crate::auth::resume::PkceCodeVerifierParser;

    #[test]
    fn a_refused_verifier_is_not_echoed() {
        let cmd = Command::new("ortie");

        let secret = "pkce-secret-value-with space-over-43-characters";
        let err = PkceCodeVerifierParser.parse_ref(&cmd, None, OsStr::new(secret));
        let err = err.unwrap_err().to_string();
        assert!(!err.contains(secret), "{err}");
        assert!(err.contains("byte 0x20"), "{err}");

        let secret = "pkce-secret-value-too-short";
        let err = PkceCodeVerifierParser.parse_ref(&cmd, None, OsStr::new(secret));
        let err = err.unwrap_err().to_string();
        assert!(!err.contains(secret), "{err}");
        assert!(err.contains("length 27"), "{err}");
    }
}
