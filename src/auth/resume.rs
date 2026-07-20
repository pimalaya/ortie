//! `auth resume` subcommand: complete an OAuth grant flow.

use std::borrow::Cow;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use log::debug;
use pimalaya_cli::printer::Printer;
use secrecy::SecretString;
use serde::{
    Deserialize,
    de::value::{Error, StrDeserializer},
};
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
};

/// Resume an existing OAuth 2.0 grant flow.
///
/// Positional input is the redirected URI (authorization-code) or the
/// device code (device grant).
#[derive(Debug, Parser)]
pub struct AuthResumeCommand {
    /// Redirected URI or device code.
    #[arg(value_name = "URI|DEVICE_CODE")]
    pub input: String,

    /// CSRF state from auth get (authorization-code only).
    #[arg(long, short, value_parser = state_parser)]
    #[arg(value_name = "VALUE")]
    pub state: Option<Oauth20State>,

    /// PKCE verifier from auth get (authorization-code only).
    #[arg(long, short, value_parser = pkce_code_verifier_parser)]
    #[arg(value_name = "CODE")]
    pub pkce: Option<Oauth20PkceCodeVerifier>,

    /// Redirect URI from auth get (authorization-code only).
    #[arg(long, short, value_parser = uri_parser)]
    pub redirect_uri: Option<Url>,
}

impl AuthResumeCommand {
    /// Completes the account's configured grant into a stored token.
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        if account.grant == GrantConfig::Device {
            if self.state.is_some() || self.pkce.is_some() || self.redirect_uri.is_some() {
                bail!(
                    "The --state, --pkce and --redirect-uri flags are only valid \
		     for the authorization-code grant"
                );
            }
            let device_code = self.input.trim();
            if device_code.is_empty() {
                bail!("Missing device code");
            }
            let Some(token_endpoint) = account.token_endpoint.clone() else {
                bail!("Missing endpoints.token in the account config");
            };
            let device = Oauth20DeviceAuthSuccessParams {
                device_code: SecretString::from(device_code),
                user_code: String::new(),
                verification_uri: String::new(),
                verification_uri_complete: None,
                expires_in: 1800,
                interval: 5,
            };
            return complete_device_token_poll(printer, &mut account, &token_endpoint, &device);
        }

        let Some(token_endpoint) = account.token_endpoint.clone() else {
            bail!("Missing endpoints.token in the account config");
        };

        let redirected_uri = Url::parse(&self.input)
            .map_err(|err| anyhow!("Invalid redirected URI `{}`: {err}", self.input))?;

        let code = match Oauth20AuthParams::from(&redirected_uri).validate(self.state.as_ref()) {
            Ok(code) => code,
            Err(Oauth20AuthParamsValidationError::Server(params)) => {
                let err = anyhow!("Authorization error (code {:?})", params.error);
                return Err(match (params.error_description, params.error_uri) {
                    (None, None) => err,
                    (Some(desc), None) => anyhow!("{desc}").context(err),
                    (None, Some(uri)) => anyhow!("{uri}").context(err),
                    (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
                });
            }
            Err(Oauth20AuthParamsValidationError::StateMissing) => {
                return Err(anyhow!("Authorization response is missing state"));
            }
            Err(Oauth20AuthParamsValidationError::StateMismatch) => {
                let req = self.state.as_ref().map(|state| state.expose());
                return Err(
                    anyhow!("Request state {req:?} does not match response state")
                        .context("Authorization request and response states do not match"),
                );
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
        })?;

        match res {
            Ok(res) => report_token_issued(printer, &mut account, &res),
            Err(res) => {
                debug!("execute issue access token error hook");
                account.execute_on_issue_error_hook(&res);
                let err = anyhow!("Issue access token error (code {:?})", res.error);
                Err(match (res.error_description, res.error_uri) {
                    (None, None) => err,
                    (Some(desc), None) => anyhow!("{desc}").context(err),
                    (None, Some(uri)) => anyhow!("{uri}").context(err),
                    (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
                })
            }
        }
    }
}

pub fn uri_parser(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|err| err.to_string())
}

pub fn state_parser(state: &str) -> Result<Oauth20State, String> {
    Oauth20State::deserialize(StrDeserializer::<Error>::new(state)).map_err(|e| e.to_string())
}

pub fn pkce_code_verifier_parser(verifier: &str) -> Result<Oauth20PkceCodeVerifier, String> {
    verifier
        .parse()
        .map_err(|b| format!("Invalid 0x{b:x} found in PKCE code verifier: {verifier}"))
}
