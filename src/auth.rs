//! # Auth commands
//!
//! The `auth` subcommand tree, obtaining OAuth 2.0 access tokens by
//! running the grant configured on the account.

pub mod get;
pub mod handoff;
pub mod listen;
pub mod resume;

use anyhow::{Result, bail};
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    account::Account,
    auth::{
        get::AuthGetCommand, handoff::AuthHandoff, listen::AuthListenCommand,
        resume::AuthResumeCommand,
    },
    config::GrantConfig,
};

/// Get a fresh access token by running the account's OAuth grant.
///
/// Start an authorization-code, device or client-credentials grant, listen
/// for the redirection of a handed-off one, or resume one with a redirected
/// URI or a device code.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    Get(AuthGetCommand),
    Listen(AuthListenCommand),
    #[command(visible_alias = "continue")]
    Resume(AuthResumeCommand),
}

impl AuthCommand {
    /// Dispatches the auth leaf on the resolved account, `invocation`
    /// starting the commands an `auth get` handoff prints.
    ///
    /// `listen` and `resume` read the handoff `--handoff` names, on an
    /// authorization code account only, stdin when `stdin_handoff` allows
    /// it, which a REPL session does not. An interrupted `get` prints its
    /// handoff when `interrupt_handoff` allows it, which a REPL session
    /// does not either.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        account: &mut Account,
        invocation: &str,
        (stdin_handoff, interrupt_handoff): (bool, bool),
    ) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account, invocation, interrupt_handoff),
            Self::Listen(mut cmd) => {
                if let Some(source) = cmd.handoff.take() {
                    let handoff = read_handoff(account, &source, stdin_handoff)?;
                    handoff.fill(&mut cmd.state, &mut cmd.pkce, &mut cmd.redirect_uri)?;
                }

                cmd.execute(printer, account)
            }
            Self::Resume(mut cmd) => {
                if let Some(source) = cmd.handoff.take() {
                    let handoff = read_handoff(account, &source, stdin_handoff)?;
                    handoff.fill(&mut cmd.state, &mut cmd.pkce, &mut cmd.redirect_uri)?;
                }

                cmd.execute(printer, account)
            }
        }
    }
}

/// Reads the handoff at `source` for `account`, refused on any grant but
/// the authorization code one.
fn read_handoff(account: &Account, source: &str, stdin_allowed: bool) -> Result<AuthHandoff> {
    if account.grant != GrantConfig::AuthorizationCode {
        bail!("The --handoff flag is only valid for the authorization-code grant");
    }

    AuthHandoff::read(source, stdin_allowed)
}
