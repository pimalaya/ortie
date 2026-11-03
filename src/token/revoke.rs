//! # Token revoke command
//!
//! The `token revoke` subcommand, telling the authorization server the
//! stored token is no longer needed (RFC 7009).

use std::fmt;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use io_oauth::{client::Oauth20ClientStd, rfc7009::revoke::*};
use pimalaya_cli::printer::Printer;
use pimalaya_config::secret::Secret;
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::account::Account;

/// Revoke the stored token at the authorization server.
///
/// Revokes the refresh token when one is stored, which ends the grant
/// behind it, else the access token. Storage is left as it is: the
/// revoked token no longer works, and the next `auth get` replaces it.
#[derive(Debug, Parser)]
pub struct TokenRevokeCommand;

impl TokenRevokeCommand {
    /// Revokes the stored token against `endpoints.revocation`, then
    /// reports which one went.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        let Some(endpoint) = account.revocation_endpoint.clone() else {
            bail!("Missing endpoints.revocation in the account config");
        };

        let token = account.resolve_token()?;

        // NOTE: RFC 7009 section 2.1 recommends the refresh token, whose
        // revocation also invalidates the access tokens of its grant.
        let (revoked, secret, hint) = match &token.refresh_token {
            Some(refresh) => (
                TokenKind::RefreshToken,
                refresh.expose_secret().to_string(),
                Oauth20TokenTypeHint::RefreshToken,
            ),
            None => (
                TokenKind::AccessToken,
                token.access_token.expose_secret().to_string(),
                Oauth20TokenTypeHint::AccessToken,
            ),
        };

        let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;

        let mut client =
            Oauth20ClientStd::connect(endpoint.clone(), &account.tls, account.client_id.clone())?;
        client.client_secret = client_secret;

        let mut params = Oauth20TokenRevokeParams::new(&account.client_id, secret);
        params.token_type_hint = Some(hint);

        match client.revoke_token(&endpoint, params)? {
            Ok(()) => printer.out(TokenRevokeOutput { revoked }),
            Err(res) => {
                let err = anyhow!("Revoke token error (code {:?})", res.error);

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

/// The receipt of a revocation: which stored token went.
#[derive(Clone, Copy, Debug, Eq, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRevokeOutput {
    /// The kind of the revoked token.
    pub revoked: TokenKind,
}

/// The kind of a stored token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenKind {
    /// The access token, revoked when no refresh token is stored.
    AccessToken,
    /// The refresh token, revoking its grant.
    RefreshToken,
}

impl fmt::Display for TokenRevokeOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.revoked {
            TokenKind::AccessToken => writeln!(f, "Access token successfully revoked"),
            TokenKind::RefreshToken => writeln!(f, "Refresh token successfully revoked"),
        }
    }
}
