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

use std::{
    borrow::Cow,
    collections::HashSet,
    fmt,
    io::{stdout, BufRead, BufReader, IsTerminal, Write},
    net::{Shutdown, TcpListener},
};

use anyhow::Result;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use io_oauth::v2_0::authorization_code_grant::{
    authorization_request::AuthorizationRequestParams,
    pkce::{PkceCodeChallenge, PkceCodeVerifier},
    state::State,
};
use pimalaya_toolbox::terminal::printer::Printer;
use serde::{
    de::value::{Error, StrDeserializer, StringDeserializer},
    Deserialize, Serialize, Serializer,
};
use url::Url;

use crate::{account::Account, auth::resume::ResumeAuthorizationCommand};

/// Initiate a new OAuth 2.0 Authorization Code Grant from scratch.
///
/// If this command is used in an interactive shell, a fake redirect
/// server is spawned in order to intercept the OAuth 2.0 redirection.
#[derive(Debug, Parser)]
pub struct GetAuthorizationCommand;

impl GetAuthorizationCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let interactive = stdout().is_terminal();

        // generate a URL-friendly state for better user
        // experience
        let state = State::default();
        let state = BASE64_URL_SAFE_NO_PAD.encode(state.expose());
        let state = State::deserialize(StringDeserializer::<Error>::new(state)).unwrap();

        let pkce_code_challenge = if account.pkce {
            Some(PkceCodeChallenge::default())
        } else {
            None
        };

        let request_params = AuthorizationRequestParams {
            client_id: account.client_id.as_str().into(),
            redirect_uri: account
                .endpoints
                .redirection
                .as_ref()
                .map(|uri| uri.as_str().into()),
            scope: HashSet::from_iter(account.scopes.iter().map(Into::into)),
            state: Some(Cow::Borrowed(&state)),
            pkce_code_challenge: pkce_code_challenge.as_ref().map(Cow::Borrowed),
        };

        let mut uri = account.endpoints.authorization.clone();
        uri.set_query(Some(&request_params.to_form_url_encoded_string()));

        let authorization_uri = AuthorizationUri {
            authorization_uri: &uri,
            state: &state,
            pkce_code_verifier: pkce_code_challenge
                .as_ref()
                .map(|challenge| &challenge.verifier),
            interactive,
        };

        if printer.is_json() || !interactive {
            return printer.out(authorization_uri);
        }

        println!("{authorization_uri}");

        if interactive {
            if let Err(err) = open::that(uri.as_str()) {
                println!("Cannot open your browser ({err})");

                let msg = "Click on the link to manually start the authorization process";
                println!("{msg}: {uri}");
            }
        }

        println!("Spawn fake HTTP redirection server…");
        let scheme = account.endpoints.redirection_scheme();
        let host = account.endpoints.redirection_host();
        let port = account.endpoints.redirection_port();
        let listener = TcpListener::bind((host, port))?;

        println!("Wait for redirection…");
        let (mut stream, _) = listener.accept()?;
        println!("Continue authorization process…");
        let mut reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        let redirected_path = request_line.split_whitespace().nth(1).unwrap();
        let redirected_uri: Url = format!("{scheme}://{host}:{port}{redirected_path}")
            .parse()
            .unwrap();

        let stream = reader.into_inner();
        stream.write_all(b"HTTP/1.0 200 OK\r\n\r\nAuthorization succeeded!")?;
        stream.shutdown(Shutdown::Both)?;

        let cmd = ResumeAuthorizationCommand {
            state: Some(state),
            pkce: pkce_code_challenge.map(|pkce| pkce.verifier),
            redirected_uri,
        };

        cmd.execute(printer, account)
    }
}

#[derive(Serialize)]
pub struct AuthorizationUri<'a> {
    authorization_uri: &'a Url,
    #[serde(serialize_with = "serialize_state")]
    state: &'a State,
    #[serde(serialize_with = "serialize_pkce_code_verifier")]
    pkce_code_verifier: Option<&'a PkceCodeVerifier>,
    interactive: bool,
}

pub fn serialize_state<S: Serializer>(state: &State, s: S) -> Result<S::Ok, S::Error> {
    let state = String::from_utf8_lossy(state.expose());
    s.serialize_str(&state)
}

pub fn serialize_pkce_code_verifier<S: Serializer>(
    verifier: &Option<&PkceCodeVerifier>,
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

impl fmt::Display for AuthorizationUri<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = String::from_utf8_lossy(self.state.expose());
        writeln!(f, "Created authorization request with:")?;
        writeln!(f, " - state: {state}")?;

        if let Some(verifier) = &self.pkce_code_verifier {
            let verifier = String::from_utf8_lossy(verifier.expose());
            writeln!(f, " - pkce: {verifier}")?;
        }

        writeln!(f)?;
        if self.interactive {
            write!(f, "Sending authorization request to your browser…")
        } else {
            let msg = "Click on the link to start the authorization process";
            write!(f, "{msg}: {}", self.authorization_uri)
        }
    }
}

pub fn url_parser(url: &str) -> Result<Url, String> {
    Url::parse(url).map_err(|err| err.to_string())
}

pub fn state_parser(state: &str) -> Result<Cow<'static, State>, String> {
    let deserializer = StrDeserializer::<Error>::new(state);
    match State::deserialize(deserializer) {
        Ok(state) => Ok(Cow::Owned(state)),
        Err(err) => Err(err.to_string()),
    }
}

pub fn pkce_code_verifier_parser(verifier: &str) -> Result<PkceCodeVerifier, String> {
    match verifier.parse() {
        Ok(state) => Ok(state),
        Err(b) => {
            let err = format!("Invalid 0x{b:x} found in PKCE code verifier: {verifier}");
            Err(err)
        }
    }
}
