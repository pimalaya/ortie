//! `auth get` subcommand: initiate a new authorization code grant flow.

use std::{
    borrow::Cow,
    collections::BTreeSet,
    fmt,
    io::{IsTerminal, stdout},
};

use anyhow::{Result, bail};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::{
    Deserialize, Serialize, Serializer,
    de::value::{Error, StringDeserializer},
};
use url::Url;

use io_oauth::{
    rfc6749::{
        auth_request::Oauth20AuthRequestParams, client::await_redirect, state::Oauth20State,
    },
    rfc7636::pkce::{
        Oauth20PkceCodeChallenge, Oauth20PkceCodeChallengeMethod, Oauth20PkceCodeVerifier,
    },
};

use crate::{
    account::Account,
    auth::resume::AuthResumeCommand,
    config::{GrantConfig, PkceConfig},
};

/// Initiate a new OAuth 2.0 Authorization Code Grant from scratch.
///
/// If this command is used in an interactive shell, a fake redirect
/// server is spawned in order to intercept the OAuth 2.0 redirection.
#[derive(Debug, Parser)]
pub struct AuthGetCommand;

impl AuthGetCommand {
    /// Runs the grant configured on the account and completes it into
    /// a stored access token (interactive shells chain into resume).
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        // FIXME: dispatch on the grant once the device authorization
        // grant path lands.
        if account.grant == GrantConfig::Device {
            bail!("The device authorization grant is not supported yet");
        }

        let Some(authorization_endpoint) = &account.authorization_endpoint else {
            bail!("Missing endpoints.authorization in the account config");
        };

        let interactive = stdout().is_terminal();

        // NOTE: re-encode the random state in URL-safe base64 so it
        // round-trips through the redirect URI without escaping.
        let state = Oauth20State::default();
        let state = BASE64_URL_SAFE_NO_PAD.encode(state.expose());
        let state = Oauth20State::deserialize(StringDeserializer::<Error>::new(state)).unwrap();

        let pkce_code_challenge = match account.pkce {
            PkceConfig::S256 => Some(Oauth20PkceCodeChallenge::default()),
            PkceConfig::Plain => Some(Oauth20PkceCodeChallenge {
                method: Oauth20PkceCodeChallengeMethod::Plain,
                verifier: Oauth20PkceCodeVerifier::default(),
            }),
            PkceConfig::Off => None,
        };

        let redirect_uri = account.redirection()?;

        let auth_uri = Oauth20AuthRequestParams {
            client_id: account.client_id.as_str().into(),
            redirect_uri: Some(Cow::from(redirect_uri.as_str())),
            scope: BTreeSet::from_iter(account.scopes.iter().map(Into::into)),
            state: Some(Cow::Borrowed(&state)),
            pkce_code_challenge: pkce_code_challenge.as_ref().map(Cow::Borrowed),
            extras: account
                .extras
                .iter()
                .map(|(key, value)| (key.as_str().into(), value.as_str().into()))
                .collect(),
        }
        .build_url(authorization_endpoint);

        let authorization_uri = AuthorizationUri {
            authorization_uri: &auth_uri,
            state: &state,
            pkce_code_verifier: pkce_code_challenge
                .as_ref()
                .map(|challenge| &challenge.verifier),
            interactive,
        };

        if printer.is_json() || !interactive {
            return printer.out(authorization_uri);
        }

        println!("{authorization_uri}");

        if interactive && let Err(err) = open::that(auth_uri.as_str()) {
            println!("Cannot open your browser ({err})");

            let msg = "Click on the link to manually start the authorization process";
            println!("{msg}: {auth_uri}");
        }

        println!("Wait for redirection…");
        let redirected_uri = await_redirect(&redirect_uri)?;

        let cmd = AuthResumeCommand {
            redirected_uri,
            state: Some(state),
            pkce: pkce_code_challenge.map(|pkce| pkce.verifier),
            redirect_uri: Some(redirect_uri.into_owned()),
        };

        cmd.execute(printer, account)
    }
}

/// Printable outcome of the flow initiation: the authorization URI
/// with the state and PKCE verifier needed to resume it later.
#[derive(Serialize)]
pub struct AuthorizationUri<'a> {
    /// The composed authorization URI to open in a browser.
    authorization_uri: &'a Url,
    /// The generated CSRF state, to pass back to auth resume.
    #[serde(serialize_with = "serialize_state")]
    state: &'a Oauth20State,
    /// The generated PKCE code verifier, to pass back to auth resume.
    #[serde(serialize_with = "serialize_pkce_code_verifier")]
    pkce_code_verifier: Option<&'a Oauth20PkceCodeVerifier>,
    /// Whether the flow was initiated from an interactive shell.
    interactive: bool,
}

/// Serializes the CSRF state as its UTF-8 string form.
pub fn serialize_state<S: Serializer>(state: &Oauth20State, s: S) -> Result<S::Ok, S::Error> {
    let state = String::from_utf8_lossy(state.expose());
    s.serialize_str(&state)
}

/// Serializes the PKCE code verifier as its UTF-8 string form.
pub fn serialize_pkce_code_verifier<S: Serializer>(
    verifier: &Option<&Oauth20PkceCodeVerifier>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match verifier {
        Some(verifier) => {
            let verifier = String::from_utf8_lossy(verifier.expose());
            s.serialize_str(&verifier)
        }
        None => s.serialize_none(),
    }
}

impl fmt::Display for AuthorizationUri<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = String::from_utf8_lossy(self.state.expose());
        writeln!(f, "Created authorization request with:")?;
        writeln!(f, " - state: {state}")?;

        if let Some(verifier) = &self.pkce_code_verifier {
            let verifier = String::from_utf8_lossy(verifier.expose());
            writeln!(f, " - pkce: {verifier}")?;
        }

        writeln!(f)?;
        if self.interactive {
            writeln!(f, "Sending authorization request to your browser:")
        } else {
            writeln!(f, "Click on the link to start the authorization process:")
        }?;

        writeln!(f, "{}", self.authorization_uri)
    }
}
