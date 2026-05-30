//! `token refresh` subcommand: refresh the current access token.

use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use humantime::format_duration;
use log::debug;
use pimalaya_cli::printer::{Message, Printer};
use secrecy::SecretBox;

use pimalaya_config::secret::Secret;

use crate::{
    cli::account::Account, client::OauthClientStd,
    issue_access_token::IssueAccessTokenSuccessParams,
    refresh_access_token::RefreshAccessTokenParams,
};

/// Refresh the current access token.
///
/// This command allows you to refresh an existing access token. It
/// may fail if the refresh token is not present or expired. In this
/// case you need to start from scratch a new authorization flow with
/// auth get.
#[derive(Debug, Parser)]
pub struct TokenRefreshCommand;

impl TokenRefreshCommand {
    pub fn execute(self, printer: &mut impl Printer, mut account: Account) -> Result<()> {
        let token = account.read_from_storage()?;

        let Some(refresh_token) = token.refresh_token else {
            bail!("Missing refresh token");
        };

        let token = Self::refresh(account, refresh_token)?;

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

    pub fn refresh(
        mut account: Account,
        refresh_token: SecretBox<str>,
    ) -> Result<IssueAccessTokenSuccessParams> {
        let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;

        let mut client = OauthClientStd::connect(
            account.token_endpoint.clone(),
            &account.tls,
            account.client_id.clone(),
        )?;
        client.client_secret = client_secret;

        let res = client.refresh_access_token(RefreshAccessTokenParams {
            client_id: account.client_id.clone(),
            refresh_token,
            scopes: account.scopes.iter().map(Into::into).collect(),
        })?;

        match res {
            Ok(mut res) => {
                if res.refresh_token.is_none() {
                    res.refresh_token = account.read_from_storage()?.refresh_token;
                }

                account.write_to_storage(&res)?;

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
