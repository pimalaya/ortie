// This file is part of Ortie, a CLI to manage OAuth 2.0 access
// tokens.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::{collections::HashSet, time::Duration};

use anyhow::{anyhow, bail, Result};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use http::{
    header::{AUTHORIZATION, HOST},
    Request,
};
use humantime::format_duration;
use io_oauth::v2_0::{
    issue_access_token::IssueAccessTokenSuccessParams,
    refresh_access_token::{
        RefreshAccessTokenParams, RefreshOauth2AccessToken, RefreshOauth2AccessTokenResult,
    },
};
use io_stream::runtimes::std::handle;
use log::debug;
use pimalaya_toolbox::{
    stream::Stream,
    terminal::printer::{Message, Printer},
};
use secrecy::{ExposeSecret, SecretBox};

use crate::account::Account;

/// Refresh the current access token.
///
/// This command allows you to refresh an existing access token. It
/// may fail if the refresh token is not present or expired. In this
/// case you need to start from scratch a new authorization flow with
/// `auth get`.
#[derive(Debug, Parser)]
pub struct RefreshTokenCommand;

impl RefreshTokenCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let token = account.storage.read()?;

        let Some(refresh_token) = token.refresh_token else {
            bail!("Missing refresh token");
        };

        let token = Self::refresh(account, refresh_token)?;

        let msg = "Access token successfully refreshed";
        let msg = match token.expires_in {
            None => "{msg} (unknown expiry)".into(),
            Some(exp) => {
                let exp = Duration::from_secs(exp as u64 + 1);
                format!("{msg} (expires in {})", format_duration(exp))
            }
        };

        printer.out(Message::new(msg))
    }

    pub fn refresh(
        account: Account,
        refresh_token: SecretBox<str>,
    ) -> Result<IssueAccessTokenSuccessParams> {
        let token_endpoint = &account.endpoints.token;
        let scheme = token_endpoint.scheme();

        let Some(host) = account.endpoints.token.host_str() else {
            bail!("Missing token endpoint host name in {token_endpoint}");
        };

        let Some(port) = account.endpoints.token.port_or_known_default() else {
            bail!("Missing token endpoint port in {token_endpoint}");
        };

        let Ok(uri) = format!("{scheme}://{host}:{port}").parse() else {
            bail!("Invalid token URI using {scheme}, {host} and {port}");
        };

        let mut stream = Stream::connect(&uri, &account.tls)?;

        let mut request =
            Request::post(account.endpoints.token.path()).header(HOST, format!("{host}:{port}"));

        if let Some(secret) = account.client_secret {
            let secret = secret.get()?;
            let creds = format!("{}:{}", account.client_id, secret.expose_secret());
            let digest = BASE64_URL_SAFE_NO_PAD.encode(creds);
            request = request.header(AUTHORIZATION, format!("Basic {digest}"));
        }

        let params = RefreshAccessTokenParams {
            client_id: account.client_id.clone(),
            refresh_token: refresh_token.clone(),
            scopes: HashSet::from_iter(account.scopes.iter().map(Into::into)),
        };

        let mut send = RefreshOauth2AccessToken::new(request, params)?;
        let mut arg = None;

        let res = loop {
            match send.resume(arg.take()) {
                RefreshOauth2AccessTokenResult::Ok(res) => break res,
                RefreshOauth2AccessTokenResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                RefreshOauth2AccessTokenResult::Err(err2) => {
                    let err = "Refresh OAuth 2.0 access token error";
                    return Err(anyhow!("{err2}").context(err));
                }
            }
        };

        match res {
            Ok(mut res) => {
                if res.refresh_token.is_none() {
                    res.refresh_token = account.storage.read()?.refresh_token;
                }

                account.storage.write(&res)?;

                debug!("execute refresh access token success hook");
                account.on_refresh_access_token.execute_success(&res);

                Ok(res)
            }
            Err(res) => {
                debug!("execute refresh access token error hook");
                account.on_refresh_access_token.execute_error(&res);

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
