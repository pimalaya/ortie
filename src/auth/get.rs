//! `auth get` subcommand: initiate a new OAuth grant flow.

use std::{
    borrow::Cow,
    collections::BTreeSet,
    fmt, fs,
    io::{IsTerminal, stdout},
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
};
use clap::Parser;
use humantime::format_duration;
use log::debug;
use pimalaya_cli::printer::{Message, Printer};
use secrecy::ExposeSecret;
use serde::{
    Deserialize, Serialize, Serializer,
    de::value::{Error, StringDeserializer},
};
use url::{Host, Url};

use pimalaya_config::secret::Secret;

use io_oauth::{
    client::{Oauth20ClientStd, Oauth20ClientStdError, await_redirect},
    rfc6749::{
        auth_request::Oauth20AuthRequestParams,
        client_credentials::Oauth20ClientCredentialsRequestParams,
        issue_access_token::{
            Oauth20AccessTokenErrorCode, Oauth20AccessTokenErrorParams,
            Oauth20AccessTokenSuccessParams,
        },
        state::Oauth20State,
    },
    rfc7523::{
        assertion::{Oauth20JwtBearerClaims, Oauth20JwtBearerKey, x5t_from_der},
        client_auth::Oauth20JwtBearerClientCredentialsRequestParams,
    },
    rfc7636::pkce::{
        Oauth20PkceCodeChallenge, Oauth20PkceCodeChallengeMethod, Oauth20PkceCodeVerifier,
    },
    rfc8628::auth::{Oauth20DeviceAuthRequestParams, Oauth20DeviceAuthSuccessParams},
};

use crate::{
    account::Account,
    auth::resume::AuthResumeCommand,
    config::{GrantConfig, PkceConfig},
};

/// Initiate a new OAuth 2.0 grant from scratch.
///
/// Runs the grant configured on the account: `authorization-code`,
/// `device`, `client-credentials` or `client-credentials-jwt`.
/// Interactive shells complete the flow; non-interactive and `--json`
/// hand off to `auth resume`. The client credentials kinds complete
/// headlessly in one shot.
#[derive(Debug, Parser)]
pub struct AuthGetCommand;

impl AuthGetCommand {
    /// Runs the grant configured on the account and completes it into
    /// a stored access token (interactive shells chain into resume).
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        if account.grant == GrantConfig::Device {
            return execute_device(printer, account);
        }

        if account.grant.is_client_credentials() {
            return execute_client_credentials(printer, account);
        }

        let Some(authorization_endpoint) = &account.authorization_endpoint else {
            bail!("Missing endpoints.authorization in the account config");
        };

        let interactive = stdout().is_terminal();

        // NOTE: re-encode the random state in URL-safe base64 so it
        // round-trips through the redirect URI without escaping.
        let state = Oauth20State::default();
        let state = BASE64_URL_SAFE_NO_PAD.encode(state.expose());
        let state = Oauth20State::deserialize(StringDeserializer::<Error>::new(state)).unwrap();

        let pkce_code_challenge = match account.pkce {
            PkceConfig::S256 => Some(Oauth20PkceCodeChallenge::default()),
            PkceConfig::Plain => Some(Oauth20PkceCodeChallenge {
                method: Oauth20PkceCodeChallengeMethod::Plain,
                verifier: Oauth20PkceCodeVerifier::default(),
            }),
            PkceConfig::Off => None,
        };

        let redirect_uri = account.redirection()?;

        let auth_uri = Oauth20AuthRequestParams {
            client_id: account.client_id.as_str().into(),
            redirect_uri: Some(Cow::from(redirect_uri.as_str())),
            scope: BTreeSet::from_iter(account.scopes.iter().map(Into::into)),
            state: Some(Cow::Borrowed(&state)),
            pkce_code_challenge: pkce_code_challenge.as_ref().map(Cow::Borrowed),
            extras: account
                .extras
                .iter()
                .map(|(key, value)| (key.as_str().into(), value.as_str().into()))
                .collect(),
        }
        .build_url(authorization_endpoint);

        let authorization_uri = AuthorizationUri {
            authorization_uri: &auth_uri,
            state: &state,
            pkce_code_verifier: pkce_code_challenge
                .as_ref()
                .map(|challenge| &challenge.verifier),
            interactive,
        };

        // NOTE: non-interactive or JSON: print (or serialize) the request
        // and hand off to a manual `auth resume`. JSON stays a clean
        // structured object carrying the state and verifier, so only
        // the human output appends the ready-to-run command.
        if printer.is_json() || !interactive {
            printer.out(authorization_uri)?;

            if !printer.is_json() {
                println!();
                print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));
            }

            return Ok(());
        }

        println!("{authorization_uri}");

        if interactive && let Err(err) = open::that(auth_uri.as_str()) {
            println!("Cannot open your browser ({err})");

            let msg = "Click on the link to manually start the authorization process";
            println!("{msg}: {auth_uri}");
        }

        // NOTE: a redirection the local listener cannot bind (a reverse-DNS
        // private-use scheme, as Fastmail's dynamic registration
        // mandates) dead-ends in the browser: hand off to a manual
        // `auth resume` rather than binding a listener that would fail
        // on the unknown scheme (no host, no inferable port).
        if !is_loopback_redirect(&redirect_uri) {
            println!();
            println!(
                "Ortie cannot capture the redirection {} automatically.",
                redirect_uri.as_str(),
            );
            print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));

            return Ok(());
        }

        println!("Wait for redirection…");

        let redirected_uri = match await_redirect(&redirect_uri) {
            Ok(redirected_uri) => redirected_uri,
            // NOTE: the listener could not bind or accept (a privileged or
            // taken port, a closed browser): fall back to the manual
            // flow instead of aborting the whole grant.
            Err(err) => {
                println!();
                println!("Ortie could not capture the redirection automatically ({err}).");
                print_manual_resume(&state, pkce_code_challenge.as_ref().map(|c| &c.verifier));

                return Ok(());
            }
        };

        let cmd = AuthResumeCommand {
            input: redirected_uri.to_string(),
            state: Some(state),
            pkce: pkce_code_challenge.map(|pkce| pkce.verifier),
            redirect_uri: Some(redirect_uri.into_owned()),
        };

        cmd.execute(printer, account)
    }
}

/// Prints the manual `auth resume` command that finishes the flow by
/// hand, filled with the flow's state and (when PKCE is enabled) code
/// verifier. Used whenever the local listener cannot capture the
/// redirect: a non-interactive shell, a private-use redirection
/// scheme, or a listener that failed to bind.
///
/// The values are attached to their flag with `=` and single quoted.
/// Both halves matter: a PKCE verifier is drawn from the RFC 7636
/// unreserved set and a state from URL-safe base64, so either can
/// begin with `-` and be read as a flag rather than a value, which the
/// `=` form settles before the parser sees it; the quotes keep the
/// shell from expanding a leading `~`.
fn print_manual_resume(state: &Oauth20State, pkce: Option<&Oauth20PkceCodeVerifier>) {
    let state = shell_single_quote(&String::from_utf8_lossy(state.expose()));

    println!(
        "Once authorized, copy the URL your browser was redirected to, \
	 then run the resume subcommand:"
    );
    println!();

    match pkce {
        Some(verifier) => {
            let verifier = shell_single_quote(&String::from_utf8_lossy(verifier.expose()));
            println!("> ortie auth resume --state={state} --pkce={verifier} <REDIRECTED_URI>");
        }
        None => {
            println!("> ortie auth resume --state={state} <REDIRECTED_URI>");
        }
    }
}

/// Whether the redirection can be serviced by the local listener:
/// an http(s) URL bound to a loopback host. Any other redirection (a
/// reverse-DNS private-use scheme, as Fastmail's dynamic registration
/// mandates, or a remote host) dead-ends in the browser, so the flow
/// finishes by hand through `auth resume`.
fn is_loopback_redirect(uri: &Url) -> bool {
    let http_scheme = matches!(uri.scheme(), "http" | "https");
    let loopback_host = match uri.host() {
        Some(Host::Domain(domain)) => domain == "localhost",
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };

    http_scheme && loopback_host
}

/// Printable outcome of the flow initiation: the authorization URI
/// with the state and PKCE verifier needed to resume it later.
#[derive(Serialize)]
pub struct AuthorizationUri<'a> {
    /// The composed authorization URI to open in a browser.
    authorization_uri: &'a Url,
    /// The generated CSRF state, to pass back to auth resume.
    #[serde(serialize_with = "serialize_state")]
    state: &'a Oauth20State,
    /// The generated PKCE code verifier, to pass back to auth resume.
    #[serde(serialize_with = "serialize_pkce_code_verifier")]
    pkce_code_verifier: Option<&'a Oauth20PkceCodeVerifier>,
    /// Whether the flow was initiated from an interactive shell.
    interactive: bool,
}

/// Serializes the CSRF state as its UTF-8 string form.
pub fn serialize_state<S: Serializer>(state: &Oauth20State, s: S) -> Result<S::Ok, S::Error> {
    let state = String::from_utf8_lossy(state.expose());
    s.serialize_str(&state)
}

/// Serializes the PKCE code verifier as its UTF-8 string form.
pub fn serialize_pkce_code_verifier<S: Serializer>(
    verifier: &Option<&Oauth20PkceCodeVerifier>,
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
            writeln!(f, "Sending authorization request to your browser:")
        } else {
            writeln!(f, "Click on the link to start the authorization process:")
        }?;

        writeln!(f, "{}", self.authorization_uri)
    }
}

fn execute_device(printer: &mut impl Printer, account: &mut Account) -> Result<()> {
    let Some(device_endpoint) = account.device_authorization_endpoint.clone() else {
        bail!("Missing endpoints.device-authorization in the account config");
    };
    let Some(token_endpoint) = account.token_endpoint.clone() else {
        bail!("Missing endpoints.token in the account config");
    };

    let interactive = stdout().is_terminal();
    let params = Oauth20DeviceAuthRequestParams {
        client_id: account.client_id.as_str().into(),
        scope: BTreeSet::from_iter(account.scopes.iter().map(|s| Cow::from(s.as_str()))),
    };

    let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;
    let mut device_client = Oauth20ClientStd::connect(
        device_endpoint.clone(),
        &account.tls,
        account.client_id.clone(),
    )?;
    device_client.client_secret = client_secret;

    let device = match device_client.request_device_auth(&device_endpoint, params)? {
        Ok(device) => device,
        Err(res) => {
            debug!("execute issue access token error hook");
            account.execute_on_issue_error_hook(&res);
            let err = anyhow!("Device authorization error (code {:?})", res.error);
            return Err(match (res.error_description, res.error_uri) {
                (None, None) => err,
                (Some(desc), None) => anyhow!("{desc}").context(err),
                (None, Some(uri)) => anyhow!("{uri}").context(err),
                (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
            });
        }
    };

    let view = DeviceAuthorization {
        device_code: device.device_code.expose_secret().to_owned(),
        user_code: device.user_code.clone(),
        verification_uri: device.verification_uri.clone(),
        verification_uri_complete: device.verification_uri_complete.clone(),
        expires_in: device.expires_in,
        interval: device.interval,
        interactive,
    };

    // NOTE: D5, non-interactive or --json print and hand off to auth resume.
    if printer.is_json() || !interactive {
        printer.out(&view)?;
        if !printer.is_json() {
            println!();
            println!("Once authorized, run:");
            println!();
            println!(
                "> ortie auth resume {}",
                shell_single_quote(&view.device_code)
            );
        }
        return Ok(());
    }

    println!("{view}");
    let open_uri = device
        .verification_uri_complete
        .as_deref()
        .unwrap_or(device.verification_uri.as_str());
    if let Err(err) = open::that(open_uri) {
        println!("Cannot open your browser ({err})");
        println!("Open {open_uri} and enter the code {}", device.user_code);
    }
    println!("Waiting for authorization…");
    complete_device_token_poll(printer, account, &token_endpoint, &device)
}

/// Polls the token endpoint until the device grant completes, then stores
/// the token and fires on-issue hooks (shared with the code grant path).
pub(crate) fn complete_device_token_poll(
    printer: &mut impl Printer,
    account: &mut Account,
    token_endpoint: &Url,
    device: &Oauth20DeviceAuthSuccessParams,
) -> Result<()> {
    let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;
    let mut client = Oauth20ClientStd::connect(
        token_endpoint.clone(),
        &account.tls,
        account.client_id.clone(),
    )?;
    client.client_secret = client_secret;

    // NOTE: outer Result is transport / client-side; inner is the token body.
    match client.await_device_access_token(&account.tls, device) {
        Ok(Ok(res)) => report_token_issued(printer, account, &res),
        Ok(Err(res)) => {
            debug!("execute issue access token error hook");
            account.execute_on_issue_error_hook(&res);
            let err = anyhow!("Issue access token error (code {:?})", res.error);
            Err(match (res.error_description, res.error_uri) {
                (None, None) => err,
                (Some(desc), None) => anyhow!("{desc}").context(err),
                (None, Some(uri)) => anyhow!("{uri}").context(err),
                (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
            })
        }
        Err(err) => {
            if let Some(params) = device_poll_client_error_hook_params(&err) {
                debug!("execute issue access token error hook");
                account.execute_on_issue_error_hook(&params);
            }
            Err(err.into())
        }
    }
}

/// Local poll deadline is the client twin of server `expired_token`.
fn device_poll_client_error_hook_params(
    err: &Oauth20ClientStdError,
) -> Option<Oauth20AccessTokenErrorParams> {
    match err {
        Oauth20ClientStdError::DeviceCodeExpired => Some(Oauth20AccessTokenErrorParams {
            error: Oauth20AccessTokenErrorCode::ExpiredToken,
            error_description: Some(
                "device code expired before the user completed authorization".into(),
            ),
            error_uri: None,
        }),
        _ => None,
    }
}

/// Lifetime of a freshly minted JWT client assertion. Short by
/// design: the assertion only needs to survive one token request, and
/// a narrow window limits replay.
const JWT_ASSERTION_VALIDITY: Duration = Duration::from_secs(600);

/// Runs the client credentials grant headlessly in one shot: no
/// browser, no user code, no resume. Fires the on-issue hooks and
/// persists the token like the interactive grants.
fn execute_client_credentials(printer: &mut impl Printer, account: &mut Account) -> Result<()> {
    match request_client_credentials_token(account)? {
        Ok(res) => report_token_issued(printer, account, &res),
        Err(res) => {
            debug!("execute issue access token error hook");
            account.execute_on_issue_error_hook(&res);
            Err(client_credentials_error(
                account.grant,
                "Issue access token error",
                res,
            ))
        }
    }
}

/// Runs the configured client credentials exchange against the token
/// endpoint and returns the raw token response: Basic auth from the
/// client secret on the plain kind, a freshly minted JWT assertion on
/// the JWT kind. Shared by `auth get` (issue) and the token
/// re-acquisition path (refresh).
pub(crate) fn request_client_credentials_token(
    account: &mut Account,
) -> Result<Result<Oauth20AccessTokenSuccessParams, Oauth20AccessTokenErrorParams>> {
    let Some(token_endpoint) = account.token_endpoint.clone() else {
        bail!("Missing endpoints.token in the account config");
    };

    let mut client =
        Oauth20ClientStd::connect(token_endpoint, &account.tls, account.client_id.clone())?;

    if account.grant == GrantConfig::ClientCredentialsJwt {
        let Some(key_path) = account.client_key.clone() else {
            bail!("Missing client-key in the account config");
        };

        // NOTE: everything is re-derived at every mint: the key
        // re-read from disk, the x5t recomputed from the certificate,
        // fresh iat/exp on a short validity and a unique jti. The
        // assertion lives only for this request and is never stored.
        let pem = fs::read_to_string(&key_path)
            .with_context(|| format!("Read client key from {}", key_path.display()))?;
        let key = Oauth20JwtBearerKey::from_pkcs8_pem(&pem).or_else(|_| {
            Oauth20JwtBearerKey::from_pkcs1_pem(&pem)
                .context("Parse client key as a PKCS#8 or PKCS#1 PEM private key")
        })?;

        let x5t = match &account.client_certificate {
            None => None,
            Some(path) => {
                let bytes = fs::read(path)
                    .with_context(|| format!("Read client certificate from {}", path.display()))?;
                Some(x5t_from_der(&certificate_der(bytes)?))
            }
        };

        // NOTE: reuse the CSRF state generator as the random source
        // for the unique jti, re-encoded in URL-safe base64.
        let jti = BASE64_URL_SAFE_NO_PAD.encode(Oauth20State::default().expose());

        let claims = Oauth20JwtBearerClaims {
            iss: account.client_id.as_str().into(),
            sub: Some(account.client_id.as_str().into()),
            jti: Some(jti.into()),
            ..Default::default()
        };

        let assertion = client.sign_jwt_bearer_assertion(
            &key,
            claims,
            x5t.as_deref(),
            JWT_ASSERTION_VALIDITY,
        )?;

        // NOTE: the assertion authenticates the client, so the client
        // secret stays unset: the Basic header it would produce
        // conflicts with the assertion authentication.
        let res = client.request_jwt_bearer_client_credentials(
            Oauth20JwtBearerClientCredentialsRequestParams {
                client_id: Some(account.client_id.as_str().into()),
                scope: account.scopes.iter().map(Into::into).collect(),
                client_assertion: assertion,
            },
        )?;

        Ok(res)
    } else {
        let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;
        client.client_secret = client_secret;

        let res = client.request_client_credentials(Oauth20ClientCredentialsRequestParams {
            scope: account.scopes.iter().map(Into::into).collect(),
        })?;

        Ok(res)
    }
}

/// Returns the DER bytes of a PEM or DER encoded certificate,
/// decoding the PEM armor when present.
fn certificate_der(bytes: Vec<u8>) -> Result<Vec<u8>> {
    let Ok(text) = core::str::from_utf8(&bytes) else {
        return Ok(bytes);
    };

    if !text.contains("-----BEGIN") {
        return Ok(bytes);
    }

    let body: String = text
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .map(str::trim)
        .collect();

    BASE64_STANDARD
        .decode(body)
        .context("Decode PEM client certificate body")
}

/// Maps a client credentials error response into the reported error,
/// hinting at certificate renewal when a JWT-authenticated client is
/// rejected as invalid_client, the typical symptom of an expired or
/// revoked certificate credential.
pub(crate) fn client_credentials_error(
    grant: GrantConfig,
    action: &str,
    res: Oauth20AccessTokenErrorParams,
) -> anyhow::Error {
    let Oauth20AccessTokenErrorParams {
        error,
        error_description,
        error_uri,
    } = res;

    let hint = (grant == GrantConfig::ClientCredentialsJwt
        && error == Oauth20AccessTokenErrorCode::InvalidClient)
        .then_some("The certificate credential may be expired and need renewal");

    let top = format!("{action} (code {error:?})");
    let detail = match (error_description, error_uri) {
        (None, None) => None,
        (Some(desc), None) => Some(desc),
        (None, Some(uri)) => Some(uri),
        (Some(desc), Some(uri)) => Some(format!("{desc}: {uri}")),
    };

    match (hint, detail) {
        (None, None) => anyhow!("{top}"),
        (None, Some(detail)) => anyhow!("{detail}").context(top),
        (Some(hint), None) => anyhow!("{hint}").context(top),
        (Some(hint), Some(detail)) => anyhow!("{hint}").context(detail).context(top),
    }
}

/// Persist the token, fire on-issue success hooks, print the success
/// message. Shared by the authorization-code and device grants.
pub(crate) fn report_token_issued(
    printer: &mut impl Printer,
    account: &mut Account,
    res: &Oauth20AccessTokenSuccessParams,
) -> Result<()> {
    account.write_to_storage(res.clone())?;
    debug!("execute issue access token success hook");
    account.execute_on_issue_success_hook(res);
    let msg = "Access token successfully issued";
    let msg = match res.expires_in {
        None => format!("{msg} (unknown expiry)"),
        Some(exp) => {
            let exp = Duration::from_secs(exp as u64 + 1);
            format!("{msg} (expires in {})", format_duration(exp))
        }
    };
    printer.out(Message::new(msg))
}

/// Single-quote for safe paste into a POSIX shell command line.
fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Printable / JSON device-authorization response for resume handoff.
#[derive(Serialize)]
struct DeviceAuthorization {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: usize,
    interval: usize,
    interactive: bool,
}

impl fmt::Display for DeviceAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created device authorization request with:")?;
        writeln!(f, " - user code: {}", self.user_code)?;
        writeln!(f, " - verification URI: {}", self.verification_uri)?;
        if let Some(uri) = &self.verification_uri_complete {
            writeln!(f, " - complete URI: {uri}")?;
        }
        // NOTE: interactive sessions poll in-process; hide the device code.
        if !self.interactive {
            writeln!(f, " - device code: {}", self.device_code)?;
        }
        writeln!(f, " - expires in: {}s", self.expires_in)?;
        writeln!(f, " - interval: {}s", self.interval)?;
        writeln!(f)?;
        writeln!(
            f,
            "Navigate to {} and enter the code {}",
            self.verification_uri, self.user_code
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_code_expired_maps_to_expired_token_hook_params() {
        let params =
            device_poll_client_error_hook_params(&Oauth20ClientStdError::DeviceCodeExpired)
                .expect("DeviceCodeExpired must fire on-issue error hook");
        assert_eq!(params.error, Oauth20AccessTokenErrorCode::ExpiredToken);
    }

    #[test]
    fn network_client_errors_do_not_synthesize_hook_params() {
        let io_err = Oauth20ClientStdError::Io(std::io::Error::other("connection reset"));
        assert!(device_poll_client_error_hook_params(&io_err).is_none());
    }

    #[test]
    fn shell_single_quote_escapes_embedded_quotes() {
        assert_eq!(shell_single_quote("plain"), "'plain'");
        assert_eq!(shell_single_quote("a;b"), "'a;b'");
        assert_eq!(shell_single_quote("it's"), "'it'\\''s'");
    }
}
