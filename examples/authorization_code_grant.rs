use std::{
    borrow::Cow,
    env,
    io::{stdin, stdout, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use io_http::rfc9110::request::HttpRequest;
use ortie::authorization_code_grant::{
    access_token_request::{
        AccessTokenRequestParams, RequestOauth2AccessToken, RequestOauth2AccessTokenResult,
    },
    authorization_request::AuthorizationRequestParams,
    authorization_response::AuthorizeParams,
    state::State,
};
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::ConfigVerifierExt;
use secrecy::ExposeSecret;
use url::Url;

fn main() {
    env_logger::init();

    let client_id = match env::var("CLIENT_ID") {
        Ok(id) => id,
        Err(_) => read_line("Client ID?"),
    };

    let redirect_uri = match env::var("REDIRECT_URI") {
        Ok(uri) => uri,
        Err(_) => read_line("Redirect URI?"),
    };

    let scope = match env::var("SCOPE") {
        Ok(scopes) => scopes,
        Err(_) => read_line("Scope?"),
    };

    let mut auth_uri: Url = match env::var("AUTHORIZATION_URI") {
        Ok(url) => url.parse().unwrap(),
        Err(_) => read_line("Authorization URL?").parse().unwrap(),
    };

    let token_uri: Url = match env::var("TOKEN_URI") {
        Ok(url) => url.parse().unwrap(),
        Err(_) => read_line("Token URL?").parse().unwrap(),
    };

    let mut stream = connect(&token_uri);

    // 1. authorization request: build URL for user to browse

    let request_params = AuthorizationRequestParams {
        client_id: client_id.as_str().into(),
        redirect_uri: Some(redirect_uri.clone().into()),
        scope: scope.split_whitespace().map(Into::into).collect(),
        state: Some(Cow::Owned(State::default())),
        #[cfg(feature = "pkce")]
        pkce_code_challenge: None,
    };

    auth_uri.set_query(Some(&request_params.to_form_url_encoded_string()));

    println!();
    println!("Navigate to the following URI: {auth_uri}");
    println!();

    // 2. authorization response: extract code, check states

    let redirected_uri: Url = read_line("Redirected URI?").parse().unwrap();
    println!();

    let response_params = AuthorizeParams::from(&redirected_uri);

    let AuthorizeParams::Success(response_params) = response_params else {
        panic!("invalid response params");
    };

    if request_params.state != response_params.state {
        panic!("states mismatch");
    }

    // 3. access token request: send request

    let host = token_uri.host_str().unwrap();
    let port = token_uri.port_or_known_default().unwrap();
    let request = HttpRequest {
        method: "POST".into(),
        url: token_uri.clone(),
        headers: Vec::new(),
        body: Vec::new(),
    }
    .header("Host", format!("{host}:{port}"));

    let params = AccessTokenRequestParams {
        code: response_params.code,
        redirect_uri: Some(redirect_uri.into()),
        client_id: client_id.into(),
        #[cfg(feature = "pkce")]
        pkce_code_verifier: None,
    };

    let mut send = RequestOauth2AccessToken::new(request, params);
    let mut buf = [0u8; 4096];
    let mut arg: Option<&[u8]> = None;

    let res = loop {
        match send.resume(arg.take()) {
            RequestOauth2AccessTokenResult::Ok(res) => break res,
            RequestOauth2AccessTokenResult::WantsRead => {
                let n = stream.read(&mut buf).unwrap();
                arg = Some(&buf[..n]);
            }
            RequestOauth2AccessTokenResult::WantsWrite(bytes) => stream.write_all(&bytes).unwrap(),
            RequestOauth2AccessTokenResult::Err(err) => panic!("send request error: {err}"),
        }
    };

    // 4. access token response: extract access token and potential
    // refresh token

    match res {
        Ok(res) => {
            println!("access token: {:?}", res.access_token.expose_secret());
            println!();

            match res.refresh_token {
                Some(token) => println!("refresh token: {:?}", token.expose_secret()),
                None => println!("no refresh token"),
            };
        }
        Err(err) => {
            panic!("get access token error: {err:?}");
        }
    }
}

fn read_line(prompt: &str) -> String {
    print!("{prompt} ");
    stdout().flush().unwrap();

    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();

    line.trim().to_owned()
}

trait StreamExt: Read + Write {}
impl<T: Read + Write> StreamExt for T {}

fn connect(url: &Url) -> Box<dyn StreamExt> {
    let domain = url.domain().unwrap();

    if url.scheme().eq_ignore_ascii_case("https") {
        let config = ClientConfig::with_platform_verifier().unwrap();
        let server_name = domain.to_string().try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
        let tcp = TcpStream::connect((domain.to_string(), 443)).unwrap();
        let tls = StreamOwned::new(conn, tcp);
        Box::new(tls)
    } else {
        let tcp = TcpStream::connect((domain.to_string(), 80)).unwrap();
        Box::new(tcp)
    }
}
