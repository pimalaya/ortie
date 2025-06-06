use std::fmt;

use anyhow::{anyhow, Result};
use clap::Parser;
use http::{header::HOST, Request};
use io_oauth::v2_0::{RefreshAccessToken, RefreshAccessTokenParams};
use io_stream::runtimes::std::handle;
use log::debug;
use pimalaya_toolbox::terminal::printer::Printer;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::{account::Account, stream::Stream};

/// Show access token.
#[derive(Debug, Parser)]
pub struct ShowToken {
    #[arg(long, short = 'r')]
    pub auto_refresh: bool,
}

impl ShowToken {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut token = account.storage.read()?;

        if self.auto_refresh || account.auto_refresh {
            if let Some(refresh_token) = &token.refresh_token {
                if let Some(0) = &token.expires_in {
                    let (host, mut stream) =
                        Stream::connect(&account.endpoints.token, &account.tls)?;
                    let request = Request::post(account.endpoints.token.path()).header(HOST, host);
                    let params =
                        RefreshAccessTokenParams::new(account.client_id, refresh_token.clone());
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

                            token = res;
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
                }
            }
        }

        printer.out(AccessToken {
            access_token: token.access_token.expose_secret(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AccessToken<'a> {
    pub access_token: &'a str,
}

impl fmt::Display for AccessToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.access_token)
    }
}
