//! `token refresh` subcommand: refresh the current access token.

use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use humantime::format_duration;
use log::debug;
use pimalaya_cli::printer::{Message, Printer};
use secrecy::SecretBox;

use pimalaya_config::secret::Secret;

use io_oauth::{
    client::Oauth20ClientStd,
    rfc6749::{
        issue_access_token::Oauth20AccessTokenSuccessParams,
        refresh_access_token::Oauth20AccessTokenRefreshParams,
    },
};

use crate::{
    account::Account,
    auth::get::{client_credentials_error, request_client_credentials_token},
    config::GrantConfig,
};

/// How an expired token gets fresh again, decided per grant.
///
/// The client credentials grants issue no refresh token, so their
/// refresh is a silent re-acquisition: the grant runs again (the JWT
/// kind minting a fresh assertion). Every other grant exchanges its
/// refresh token, or keeps the stored token when none exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshAction {
    /// Exchange the refresh token against the token endpoint.
    Refresh,
    /// Re-run the configured client credentials grant.
    Reacquire,
    /// Nothing to do: no refresh token and no re-runnable grant.
    Keep,
}

/// Decides how an expired token gets fresh again: re-acquisition on
/// the client credentials kinds, refresh-token exchange where a
/// refresh token exists, nothing otherwise.
pub fn refresh_action(grant: GrantConfig, has_refresh_token: bool) -> RefreshAction {
    if grant.is_client_credentials() {
        RefreshAction::Reacquire
    } else if has_refresh_token {
        RefreshAction::Refresh
    } else {
        RefreshAction::Keep
    }
}

/// Refresh the current access token.
///
/// This command allows you to refresh an existing access token. It
/// may fail if the refresh token is not present or expired. In this
/// case you need to start from scratch a new authorization flow with
/// auth get. On a client credentials account it re-runs the grant,
/// since those issue no refresh token.
#[derive(Debug, Parser)]
pub struct TokenRefreshCommand;

impl TokenRefreshCommand {
    /// Refreshes the token per the account's grant (refresh-token
    /// exchange or client credentials re-acquisition) and reports the
    /// new expiry.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        let token = match refresh_action(account.grant, true) {
            RefreshAction::Reacquire => Self::reacquire(account)?,
            _ => {
                let token = account.resolve_token()?;

                let Some(refresh_token) = token.refresh_token else {
                    bail!("Missing refresh token");
                };

                Self::refresh(account, refresh_token)?
            }
        };

        let msg = "Access token successfully refreshed";
        let msg = match token.expires_in {
            None => format!("{msg} (unknown expiry)"),
            Some(exp) => {
                let exp = Duration::from_secs(exp as u64 + 1);
                format!("{msg} (expires in {})", format_duration(exp))
            }
        };

        printer.out(Message::new(msg))
    }

    /// Re-acquires a client credentials token by re-running the
    /// grant, persists it and fires the on-refresh hooks. The JWT
    /// kind mints a fresh assertion on every run; nothing but the
    /// token response is ever stored.
    pub fn reacquire(account: &mut Account) -> Result<Oauth20AccessTokenSuccessParams> {
        match request_client_credentials_token(account)? {
            Ok(res) => {
                let res = account.write_to_storage(res)?;

                debug!("execute refresh access token success hook");
                account.execute_on_refresh_success_hook(&res);

                Ok(res)
            }
            Err(res) => {
                debug!("execute refresh access token error hook");
                account.execute_on_refresh_error_hook(&res);

                Err(client_credentials_error(
                    account.grant,
                    "Refresh access token error",
                    res,
                ))
            }
        }
    }

    /// Runs the refresh grant against the token endpoint, persists
    /// the outcome (keeping the previous refresh token when the
    /// server omits a rotated one) and fires the on-refresh hooks.
    pub fn refresh(
        account: &mut Account,
        refresh_token: SecretBox<str>,
    ) -> Result<Oauth20AccessTokenSuccessParams> {
        let Some(token_endpoint) = account.token_endpoint.clone() else {
            bail!("Missing endpoints.token in the account config");
        };

        let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;

        let mut client =
            Oauth20ClientStd::connect(token_endpoint, &account.tls, account.client_id.clone())?;
        client.client_secret = client_secret;

        let res = client.refresh_access_token(Oauth20AccessTokenRefreshParams {
            client_id: account.client_id.clone(),
            client_secret: None,
            refresh_token,
            scopes: account.scopes.iter().map(Into::into).collect(),
        })?;

        match res {
            Ok(mut res) => {
                if res.refresh_token.is_none() {
                    res.refresh_token = account.resolve_token()?.refresh_token;
                }

                let res = account.write_to_storage(res)?;

                debug!("execute refresh access token success hook");
                account.execute_on_refresh_success_hook(&res);

                Ok(res)
            }
            Err(res) => {
                debug!("execute refresh access token error hook");
                account.execute_on_refresh_error_hook(&res);

                let err = anyhow!("Refresh access token error (code {:?})", res.error);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_credentials_kinds_reacquire_even_with_a_stored_refresh_token() {
        assert_eq!(
            refresh_action(GrantConfig::ClientCredentials, false),
            RefreshAction::Reacquire
        );
        assert_eq!(
            refresh_action(GrantConfig::ClientCredentialsJwt, false),
            RefreshAction::Reacquire
        );
        assert_eq!(
            refresh_action(GrantConfig::ClientCredentials, true),
            RefreshAction::Reacquire
        );
    }

    #[test]
    fn interactive_grants_refresh_only_with_a_refresh_token() {
        assert_eq!(
            refresh_action(GrantConfig::AuthorizationCode, true),
            RefreshAction::Refresh
        );
        assert_eq!(
            refresh_action(GrantConfig::Device, true),
            RefreshAction::Refresh
        );
        assert_eq!(
            refresh_action(GrantConfig::AuthorizationCode, false),
            RefreshAction::Keep
        );
    }
}
