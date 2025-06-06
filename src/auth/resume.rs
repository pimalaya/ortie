use std::{borrow::Cow, time::Duration};

use anyhow::{anyhow, Result};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use http::{
    header::{AUTHORIZATION, HOST},
    Request,
};
use humantime::format_duration;
use io_oauth::v2_0::authorization_code_grant::{
    access_token_request::{AccessTokenRequestParams, SendAccessTokenRequest},
    authorization_response::AuthorizeParams,
    pkce::PkceCodeVerifier,
    state::State,
};
use io_stream::runtimes::std::handle;
use log::debug;
use pimalaya_toolbox::terminal::printer::{Message, Printer};
use secrecy::ExposeSecret;
use serde::{
    de::value::{Error, StrDeserializer},
    Deserialize, Serializer,
};
use url::Url;

use crate::{account::Account, stream::Stream};

#[derive(Debug, Parser)]
pub struct ResumeAuthorization {
    #[arg(value_parser = uri_parser)]
    pub redirected_uri: Url,

    #[arg(long, short, value_parser = state_parser)]
    pub state: Option<State>,

    #[arg(long, short, value_parser = pkce_code_verifier_parser)]
    pub pkce: Option<PkceCodeVerifier>,
}

impl ResumeAuthorization {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let params = match AuthorizeParams::from(&self.redirected_uri) {
            AuthorizeParams::Success(params) => params,
            AuthorizeParams::Error(params) => {
                let err = anyhow!("Authorization error (code {:?})", params.error);
                return Err(match (params.error_description, params.error_uri) {
                    (None, None) => err,
                    (Some(desc), None) => anyhow!("{desc}").context(err),
                    (None, Some(uri)) => anyhow!("{uri}").context(err),
                    (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
                });
            }
        };

        let state = self.state.as_ref().map(Cow::Borrowed);

        if params.state != state {
            let req = self.state.as_ref().map(|state| state.expose());
            let res = params.state.as_ref().map(|state| state.expose());

            let err = anyhow!("Request state {req:?} differs from response state {res:?}")
                .context("Authorization request and response states do not match");

            return Err(err);
        };

        let (host, mut stream) = Stream::connect(&account.endpoints.token, &account.tls)?;
        let mut request = Request::post(account.endpoints.token.path()).header(HOST, host);

        if let Some(secret) = account.client_secret {
            let secret = secret.get()?;
            let creds = format!("{}:{}", account.client_id, secret.expose_secret());
            let digest = BASE64_URL_SAFE_NO_PAD.encode(creds);
            request = request.header(AUTHORIZATION, format!("Basic {digest}"));
        }

        let mut params = AccessTokenRequestParams {
            code: params.code,
            redirect_uri: None,
            client_id: account.client_id.into(),
            pkce_code_verifier: self.pkce.as_ref().map(Cow::Borrowed),
        };

        if let Some(uri) = &account.endpoints.redirection {
            params.redirect_uri = Some(uri.as_str().into());
        }

        let mut send = SendAccessTokenRequest::new(request, params)?;
        let mut arg = None;

        let res = loop {
            match send.resume(arg.take()) {
                Err(io) => arg = Some(handle(&mut stream, io)?),
                Ok(Ok(res)) => break res,
                Ok(Err(err)) => {
                    let err = anyhow!("{err}");
                    return Err(err.context("Parse access token response error"));
                }
            }
        };

        match res {
            Ok(res) => {
                account.storage.write(&res)?;

                debug!("execute issue access token success hook");
                account.on_issue_access_token.execute_success(&res);

                let msg = "Access token successfully issued";
                let msg = match res.expires_in {
                    None => "{msg} (unknown expiry)".into(),
                    Some(exp) => {
                        let exp = Duration::from_secs(exp as u64 + 1);
                        format!("{msg} (expires in {})", format_duration(exp))
                    }
                };

                printer.out(Message::new(msg))
            }
            Err(res) => {
                debug!("execute issue access token error hook");
                account.on_issue_access_token.execute_error(&res);

                let err = anyhow!("Issue access token error (code {:?})", res.error);
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

pub fn serialize_state<S: Serializer>(state: &State, s: S) -> Result<S::Ok, S::Error> {
    let state = String::from_utf8_lossy(state.expose());
    s.serialize_str(&state)
}

pub fn serialize_pkce_code_verifier<S: Serializer>(
    verifier: &Option<PkceCodeVerifier>,
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

pub fn uri_parser(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|err| err.to_string())
}

pub fn state_parser(state: &str) -> Result<State, String> {
    match State::deserialize(StrDeserializer::<Error>::new(state)) {
        Ok(state) => Ok(state),
        Err(err) => Err(err.to_string()),
    }
}

pub fn pkce_code_verifier_parser(verifier: &str) -> Result<PkceCodeVerifier, String> {
    match verifier.parse() {
        Ok(verifier) => Ok(verifier),
        Err(b) => {
            let err = format!("Invalid 0x{b:x} found in PKCE code verifier: {verifier}");
            Err(err)
        }
    }
}
