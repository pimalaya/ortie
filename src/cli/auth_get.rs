//! `auth get` subcommand: initiate a new authorization code grant flow.

use alloc::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
};
use std::{fmt, io::IsTerminal, io::stdout};

use anyhow::Result;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::{
    Deserialize, Serialize, Serializer,
    de::value::{Error, StrDeserializer, StringDeserializer},
};
use url::Url;

use crate::{
    authorization_code_grant::{
        authorization_request::AuthorizationRequestParams,
        pkce::{PkceCodeChallenge, PkceCodeVerifier},
        state::State,
    },
    cli::{account::Account, auth_resume::AuthResumeCommand},
    client,
};

/// Initiate a new OAuth 2.0 Authorization Code Grant from scratch.
///
/// If this command is used in an interactive shell, a fake redirect
/// server is spawned in order to intercept the OAuth 2.0 redirection.
#[derive(Debug, Parser)]
pub struct AuthGetCommand;

impl AuthGetCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let interactive = stdout().is_terminal();

        // NOTE: re-encode the random state in URL-safe base64 so it
        // round-trips through the redirect URI without escaping.
        let state = State::default();
        let state = BASE64_URL_SAFE_NO_PAD.encode(state.expose());
        let state = State::deserialize(StringDeserializer::<Error>::new(state)).unwrap();

        let pkce_code_challenge = if account.pkce {
            Some(PkceCodeChallenge::default())
        } else {
            None
        };

        let redirect_uri = account.redirection()?;

        let auth_uri = AuthorizationRequestParams {
            client_id: account.client_id.as_str().into(),
            redirect_uri: Some(Cow::from(redirect_uri.as_str())),
            scope: BTreeSet::from_iter(account.scopes.iter().map(Into::into)),
            state: Some(Cow::Borrowed(&state)),
            pkce_code_challenge: pkce_code_challenge.as_ref().map(Cow::Borrowed),
            extras: BTreeMap::new(),
        }
        .build_url(&account.authorization_endpoint);

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
        let redirected_uri = client::await_redirect(&redirect_uri)?;

        let cmd = AuthResumeCommand {
            redirected_uri,
            state: Some(state),
            pkce: pkce_code_challenge.map(|pkce| pkce.verifier),
            redirect_uri: Some(redirect_uri.into_owned()),
        };

        cmd.execute(printer, account)
    }
}

#[derive(Serialize)]
pub struct AuthorizationUri<'a> {
    authorization_uri: &'a Url,
    #[serde(serialize_with = "serialize_state")]
    state: &'a State,
    #[serde(serialize_with = "serialize_pkce_code_verifier")]
    pkce_code_verifier: Option<&'a PkceCodeVerifier>,
    interactive: bool,
}

pub fn serialize_state<S: Serializer>(state: &State, s: S) -> Result<S::Ok, S::Error> {
    let state = String::from_utf8_lossy(state.expose());
    s.serialize_str(&state)
}

pub fn serialize_pkce_code_verifier<S: Serializer>(
    verifier: &Option<&PkceCodeVerifier>,
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

pub fn url_parser(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|err| err.to_string())
}

pub fn state_parser(state: &str) -> Result<Cow<'static, State>, String> {
    let deserializer = StrDeserializer::<Error>::new(state);
    match State::deserialize(deserializer) {
        Ok(state) => Ok(Cow::Owned(state)),
        Err(err) => Err(err.to_string()),
    }
}

pub fn pkce_code_verifier_parser(verifier: &str) -> Result<PkceCodeVerifier, String> {
    match verifier.parse() {
        Ok(state) => Ok(state),
        Err(b) => {
            let err = format!("Invalid 0x{b:x} found in PKCE code verifier: {verifier}");
            Err(err)
        }
    }
}
