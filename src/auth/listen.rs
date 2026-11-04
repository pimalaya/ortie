//! # Auth listen command
//!
//! The `auth listen` subcommand, running the loopback listener of an
//! authorization code flow [`crate::auth::get`] handed off, then
//! completing it as [`crate::auth::resume`] does.

use std::io::{IsTerminal, stdout};

use anyhow::{Result, bail};
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
        resume::{AuthResumeCommand, pkce_code_verifier_parser, state_parser, uri_parser},
    },
    config::GrantConfig,
};

/// Wait for the redirection of a handed-off flow, then complete it.
///
/// Binds the loopback redirect URI a non-interactive or `--json` auth get
/// handed off, waits for the browser to be redirected there after
/// consent, then exchanges the code as auth resume does. Start it before
/// authorizing: a redirection arriving first fails in the browser, and
/// auth resume with the copied URL still finishes the flow.
///
/// Authorization-code grant only.
#[derive(Debug, Parser)]
pub struct AuthListenCommand {
    /// The state generated during the authorization flow initiation.
    #[arg(long, short, value_parser = state_parser)]
    #[arg(value_name = "VALUE")]
    pub state: Option<Oauth20State>,
    /// The PKCE code verifier generated during the initiation.
    ///
    /// Must be given when PKCE was enabled.
    #[arg(long, short, value_parser = pkce_code_verifier_parser)]
    #[arg(value_name = "CODE")]
    pub pkce: Option<Oauth20PkceCodeVerifier>,
    /// The redirect URI used during the initiation, the one bound.
    ///
    /// Falls back to `endpoints.redirection`. Must be an http(s) URI on a
    /// loopback host.
    #[arg(long, short, value_parser = uri_parser)]
    pub redirect_uri: Option<Url>,
}

impl AuthListenCommand {
    /// Waits for one redirection on the loopback redirect URI, then
    /// completes the flow into a stored token.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        if account.grant == GrantConfig::Device {
            bail!("The device grant has no redirection to listen for, run auth resume instead");
        }

        if account.grant.is_client_credentials() {
            bail!(
                "The client credentials grants complete in a single auth get, nothing to listen for"
            );
        }

        let Some(redirect_uri) = self
            .redirect_uri
            .or_else(|| account.redirection_endpoint.clone())
        else {
            bail!("Missing --redirect-uri, or endpoints.redirection in the account config");
        };

        if !is_loopback_redirect(&redirect_uri) {
            bail!(
                "Cannot listen on {redirect_uri}, an http(s) URI on a loopback host, \
                 run auth resume with the redirected URI instead"
            );
        }

        debug!("wait for redirection on {redirect_uri}");

        // NOTE: only a person at a terminal reads this; a script waits
        // for the receipt alone.
        if !printer.is_json() && stdout().is_terminal() {
            println!("Wait for redirection on {redirect_uri}…");
        }

        let redirected_uri = await_redirect(&redirect_uri)?;

        let cmd = AuthResumeCommand {
            input: redirected_uri.to_string(),
            state: self.state,
            pkce: self.pkce,
            redirect_uri: Some(redirect_uri),
        };

        cmd.execute(printer, account)
    }
}
