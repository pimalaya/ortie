use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use http::{
    header::{AUTHORIZATION, HOST},
    Request,
};
use humantime::format_duration;
use io_oauth::v2_0::{RefreshAccessToken, RefreshAccessTokenParams};
use io_stream::runtimes::std::handle;
use log::debug;
use pimalaya_toolbox::{
    stream::Stream,
    terminal::printer::{Message, Printer},
};
use secrecy::ExposeSecret;

use crate::account::Account;

/// Refresh access token.
#[derive(Debug, Parser)]
pub struct RefreshToken;

impl RefreshToken {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let token = account.storage.read()?;

        let Some(refresh_token) = &token.refresh_token else {
            bail!("Missing refresh token");
        };

        let token_endpoint = &account.endpoints.token;

        let Some(host) = account.endpoints.token.host_str() else {
            bail!("Missing token endpoint host name in {token_endpoint}");
        };

        let Some(port) = account.endpoints.token.port_or_known_default() else {
            bail!("Missing token endpoint port in {token_endpoint}");
        };

        let mut stream = Stream::connect(host, port, &account.tls)?;

        let mut request =
            Request::post(account.endpoints.token.path()).header(HOST, format!("{host}:{port}"));

        if let Some(secret) = account.client_secret {
            let secret = secret.get()?;
            let creds = format!("{}:{}", account.client_id, secret.expose_secret());
            let digest = BASE64_URL_SAFE_NO_PAD.encode(creds);
            request = request.header(AUTHORIZATION, format!("Basic {digest}"));
        }

        let params = RefreshAccessTokenParams::new(account.client_id, refresh_token.clone());
        let mut send = RefreshAccessToken::new(request, params)?;
        let mut arg = None;

        let res = loop {
            match send.resume(arg.take()) {
                Err(io) => arg = Some(handle(&mut stream, io)?),
                Ok(Ok(res)) => break res,
                Ok(Err(err2)) => {
                    let err = "Parse refresh token response error";
                    return Err(anyhow!(err2).context(err));
                }
            }
        };

        match res {
            Ok(res) => {
                account.storage.write(&res)?;

                debug!("execute refresh access token success hook");
                account.on_refresh_access_token.execute_success(&res);

                let msg = "Access token successfully refreshed";
                let msg = match res.expires_in {
                    None => "{msg} (unknown expiry)".into(),
                    Some(exp) => {
                        let exp = Duration::from_secs(exp as u64 + 1);
                        format!("{msg} (expires in {})", format_duration(exp))
                    }
                };

                printer.out(Message::new(msg))?;
            }
            Err(res) => {
                debug!("execute refresh access token error hook");
                account.on_refresh_access_token.execute_error(&res);

                let err = anyhow!("Refresh access token error (code {:?})", res.error);

                return Err(match (res.error_description, res.error_uri) {
                    (None, None) => err,
                    (Some(desc), None) => anyhow!("{desc}").context(err),
                    (None, Some(uri)) => anyhow!("{uri}").context(err),
                    (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
                });
            }
        }

        Ok(())
    }
}
