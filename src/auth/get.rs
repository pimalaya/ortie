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
use url::{Host, Url};

use io_oauth::{
    client::await_redirect,
    rfc6749::{auth_request::Oauth20AuthRequestParams, state::Oauth20State},
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

        // Non-interactive or JSON: print (or serialize) the request
        // and hand off to a manual `auth resume`. JSON stays a clean
        // structured object carrying the state and verifier, so only
        // the human output appends the ready-to-run command.
        if printer.is_json() || !interactive {
            printer.out(authorization_uri)?;

            if !printer.is_json() {
                println!();
                print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));
            }

            return Ok(());
        }

        println!("{authorization_uri}");

        if interactive && let Err(err) = open::that(auth_uri.as_str()) {
            println!("Cannot open your browser ({err})");

            let msg = "Click on the link to manually start the authorization process";
            println!("{msg}: {auth_uri}");
        }

        // A redirection the local listener cannot bind (a reverse-DNS
        // private-use scheme, as Fastmail's dynamic registration
        // mandates) dead-ends in the browser: hand off to a manual
        // `auth resume` rather than binding a listener that would fail
        // on the unknown scheme (no host, no inferable port).
        if !is_loopback_redirect(&redirect_uri) {
            println!();
            println!(
                "Ortie cannot capture the redirection {} automatically.",
                redirect_uri.as_str(),
            );
            print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));

            return Ok(());
        }

        println!("Wait for redirection…");

        let redirected_uri = match await_redirect(&redirect_uri) {
            Ok(redirected_uri) => redirected_uri,
            // The listener could not bind or accept (a privileged or
            // taken port, a closed browser): fall back to the manual
            // flow instead of aborting the whole grant.
            Err(err) => {
                println!();
                println!("Ortie could not capture the redirection automatically ({err}).");
                print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));

                return Ok(());
            }
        };

        let cmd = AuthResumeCommand {
            redirected_uri,
            state: Some(state),
            pkce: pkce_code_challenge.map(|pkce| pkce.verifier),
            redirect_uri: Some(redirect_uri.into_owned()),
        };

        cmd.execute(printer, account)
    }
}

/// Prints the manual `auth resume` command that finishes the flow by
/// hand, filled with the flow's state and (when PKCE is enabled) code
/// verifier. Used whenever the local listener cannot capture the
/// redirect: a non-interactive shell, a private-use redirection
/// scheme, or a listener that failed to bind.
fn print_manual_resume(state: &Oauth20State, pkce: Option<&Oauth20PkceCodeVerifier>) {
    let state = String::from_utf8_lossy(state.expose());

    println!(
        "Once authorized, copy the URL your browser was redirected to, \
	 then run the resume subcommand:"
    );
    println!();

    match pkce {
        Some(verifier) => {
            let verifier = String::from_utf8_lossy(verifier.expose());
            println!("> ortie auth resume --state {state} --pkce {verifier} <REDIRECTED_URI>");
        }
        None => {
            println!("> ortie auth resume <REDIRECTED_URI> --state {state} <REDIRECTED_URI>");
        }
    }
}

/// Whether the redirection can be serviced by the local listener:
/// an http(s) URL bound to a loopback host. Any other redirection (a
/// reverse-DNS private-use scheme, as Fastmail's dynamic registration
/// mandates, or a remote host) dead-ends in the browser, so the flow
/// finishes by hand through `auth resume`.
fn is_loopback_redirect(uri: &Url) -> bool {
    let http_scheme = matches!(uri.scheme(), "http" | "https");
    let loopback_host = match uri.host() {
        Some(Host::Domain(domain)) => domain == "localhost",
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };

    http_scheme && loopback_host
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
