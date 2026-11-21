//! # Auth listen command
//!
//! The `auth listen` subcommand, running the loopback listener of an
//! authorization code flow [`crate::auth::get`] handed off, then
//! completing it as [`crate::auth::resume`] does.

use std::io::{IsTerminal, stdout};

use anyhow::Result;
use clap::Parser;
use io_oauth::{
    client::await_redirect, rfc6749::state::Oauth20State, rfc7636::pkce::Oauth20PkceCodeVerifier,
};
use log::debug;
use pimalaya_cli::printer::Printer;
use url::Url;

use crate::{
    account::Account,
    auth::{
        get::is_loopback_redirect,
        resume::{AuthResumeCommand, PkceCodeVerifierParser, state_parser, uri_parser},
    },
    config::GrantConfig,
    error::{self, ErrorCode},
};

/// Wait for the redirection of a handed-off flow, then complete it.
///
/// Binds the loopback redirect URI a non-interactive or `--json` auth get
/// handed off, waits for the browser to be redirected there after
/// consent, then exchanges the code as auth resume does. Start it before
/// authorizing: a redirection arriving first fails in the browser, and
/// auth resume with the copied URL still finishes the flow.
///
/// Authorization-code grant only. --handoff reads the object `auth get
/// --json` printed, from a file or stdin, in place of --state and --pkce.
///
/// Prints the receipt {account, expiresIn, withRefreshToken} and exits 0.
/// Exits 2 coded usage-invalid on a command line the account refuses (a
/// device or client credentials account, a --redirect-uri that is not an
/// http loopback URI), 1 coded endpoint-missing when neither
/// --redirect-uri nor endpoints.redirection gives a redirection, 1 coded
/// redirection-unsupported when endpoints.redirection is not an http
/// loopback URI, and 1 on any other failure.
#[derive(Debug, Parser)]
pub struct AuthListenCommand {
    /// The state generated during the authorization flow initiation.
    ///
    /// A request that does not echo it is answered 404 Not Found, and the
    /// wait goes on.
    #[arg(long, short, value_parser = state_parser)]
    #[arg(value_name = "VALUE")]
    pub state: Option<Oauth20State>,
    /// The PKCE code verifier generated during the initiation.
    ///
    /// Must be given when PKCE was enabled.
    #[arg(long, short, value_parser = PkceCodeVerifierParser)]
    #[arg(value_name = "CODE")]
    pub pkce: Option<Oauth20PkceCodeVerifier>,
    /// The redirect URI used during the initiation, the one bound.
    ///
    /// Falls back to `endpoints.redirection`. Must be an http URI on a
    /// loopback host.
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

impl AuthListenCommand {
    /// Waits for one redirection on the loopback redirect URI, then
    /// completes the flow into a stored token.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        if account.grant == GrantConfig::Device {
            let message =
                "The device grant has no redirection to listen for, run auth resume instead";
            return Err(error::usage(message));
        }

        if account.grant.is_client_credentials() {
            let message = "The client credentials grants complete in a single auth get, nothing to listen for";
            return Err(error::usage(message));
        }

        let redirect_uri = match (self.redirect_uri, &account.redirection_endpoint) {
            (Some(uri), _) if !is_loopback_redirect(&uri) => {
                return Err(error::usage(format!(
                    "Cannot listen on {uri}, not an http URI on a loopback host, \
                     run auth resume with the redirected URI instead"
                )));
            }
            (Some(uri), _) => uri,
            (None, Some(uri)) if !is_loopback_redirect(uri) => {
                return Err(error::coded(
                    ErrorCode::RedirectionUnsupported,
                    format!(
                        "Cannot listen on endpoints.redirection {uri}, not an http \
                         URI on a loopback host, run auth resume with the redirected \
                         URI instead"
                    ),
                ));
            }
            (None, Some(uri)) => uri.clone(),
            (None, None) => {
                let message =
                    "Missing --redirect-uri, or endpoints.redirection in the account config";
                return Err(error::coded(ErrorCode::EndpointMissing, message));
            }
        };

        debug!("wait for redirection on {redirect_uri}");

        // NOTE: only a person at a terminal reads this; a script waits
        // for the receipt alone.
        if !printer.is_json() && stdout().is_terminal() {
            println!("Wait for redirection on {redirect_uri}…");
        }

        let redirected_uri = await_redirect(&redirect_uri, self.state.as_ref())?;

        let cmd = AuthResumeCommand {
            input: redirected_uri.to_string(),
            state: self.state,
            pkce: self.pkce,
            redirect_uri: Some(redirect_uri),
            handoff: None,
        };

        cmd.execute(printer, account)
    }
}
