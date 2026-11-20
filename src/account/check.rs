//! # Account check command
//!
//! The `account check` subcommand: whether the account's token storage
//! reads and its endpoints answer, one row each, on pimalaya-cli's shared
//! [`AccountCheckOutput`], the endpoints row naming the TLS configured.

use std::io::{Read, Write};

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_oauth::client::Oauth20ClientStd;
use log::debug;
use pimalaya_cli::{
    account::{AccountCheck, AccountCheckOutput, BackendCheck, BackendCheckTls},
    error::{PartialOutcome, with_code},
    printer::Printer,
};
use pimalaya_stream::tls::{RustlsCrypto, Tls, TlsProvider};
use url::{Position, Url};

use crate::{
    account::Account,
    config::GrantConfig,
    error::{self, ErrorCode},
};

/// The most bytes read looking for the status line of an answer.
const STATUS_LINE_MAX: usize = 1024;

/// Check that the account's token storage and endpoints work.
///
/// Prints two rows, each OK or FAIL with why. `storage` runs
/// `storage.read.command` and parses the token it prints; the write
/// command never runs, since it would replace the stored token.
/// `endpoints` checks that the endpoints the grant needs are configured,
/// then sends each configured one but the redirection a `HEAD` request
/// carrying no credential, any HTTP answer counting. Neither proves the
/// provider accepts the client: only a grant does.
///
/// A failed row exits the command with status 3, the report being the
/// output rather than a failure object.
///
/// JSON output: `{"accounts": [{"account", "backends": [{"backend", "ok",
/// "code", "error", "tls"}]}]}`, `backend` being `storage` or `endpoints`,
/// `tls` the TLS configured for the endpoints, `null` for the storage and
/// for endpoints all over plain HTTP.
#[derive(Debug, Parser)]
pub struct AccountCheckCommand;

impl AccountCheckCommand {
    /// Checks the storage, then the endpoints, of `account`, printing
    /// both outcomes whatever the first one was.
    pub fn execute(self, printer: &mut impl Printer, account: &Account) -> Result<()> {
        let storage = account.read_storage().map(|_| ());
        let endpoints = reach_endpoints(account);

        let mut endpoints = row("endpoints", endpoints);
        endpoints.tls = over_tls(account).then(|| tls_check(&account.tls));

        let report = AccountCheckOutput {
            accounts: vec![AccountCheck {
                account: account.name.clone(),
                backends: vec![row("storage", storage), endpoints],
            }],
        };

        let ok = report.ok();
        printer.out(report)?;

        if !ok {
            return Err(PartialOutcome::default().into());
        }

        Ok(())
    }
}

/// Fails on the first endpoint the grant needs that the account leaves
/// out, then on the first configured one that does not answer.
fn reach_endpoints(account: &Account) -> Result<()> {
    let endpoints = [
        ("endpoints.authorization", &account.authorization_endpoint),
        (
            "endpoints.device-authorization",
            &account.device_authorization_endpoint,
        ),
        ("endpoints.token", &account.token_endpoint),
        ("endpoints.revocation", &account.revocation_endpoint),
    ];

    let needed: &[&str] = match account.grant {
        GrantConfig::AuthorizationCode => &["endpoints.authorization", "endpoints.token"],
        GrantConfig::Device => &["endpoints.device-authorization", "endpoints.token"],
        GrantConfig::ClientCredentials | GrantConfig::ClientCredentialsJwt => &["endpoints.token"],
    };

    for (key, url) in endpoints {
        if url.is_none() && needed.contains(&key) {
            return Err(error::endpoint_missing(key));
        }
    }

    for (key, url) in endpoints {
        if let Some(url) = url {
            reach(account, url).with_context(|| format!("Cannot reach {key} at {url}"))?;
        }
    }

    Ok(())
}

/// Sends `url` one `HEAD` request, over a connection opened as the grants
/// open theirs, and reads the status line of the answer.
fn reach(account: &Account, url: &Url) -> Result<()> {
    let client = Oauth20ClientStd::connect(url.clone(), &account.tls, account.client_id.clone())?;
    let mut stream = client.stream;

    let target = &url[Position::BeforePath..Position::AfterQuery];
    let host = &url[Position::BeforeHost..Position::AfterPort];
    let request = format!("HEAD {target} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;

    let mut answer = Vec::new();
    let mut buf = [0; 256];

    while !answer.windows(2).any(|w| w == b"\r\n") && answer.len() < STATUS_LINE_MAX {
        match stream.read(&mut buf)? {
            0 => break,
            n => answer.extend_from_slice(&buf[..n]),
        }
    }

    let answer = String::from_utf8_lossy(&answer);
    let mut words = answer.lines().next().unwrap_or_default().split(' ');

    match (words.next(), words.next()) {
        (Some(version), Some(status))
            if version.starts_with("HTTP/")
                && status.len() == 3
                && status.bytes().all(|b| b.is_ascii_digit()) =>
        {
            debug!("{url} answered {status}");
            Ok(())
        }
        _ => bail!("The answer is no HTTP status line"),
    }
}

/// Whether one of the endpoints the check sends a request is `https`.
fn over_tls(account: &Account) -> bool {
    let endpoints = [
        &account.authorization_endpoint,
        &account.device_authorization_endpoint,
        &account.token_endpoint,
        &account.revocation_endpoint,
    ];

    let mut urls = endpoints.into_iter().flatten();
    urls.any(|url| url.scheme() == "https")
}

/// The TLS of the endpoints as configured, `null` where the build's
/// default answers.
fn tls_check(tls: &Tls) -> BackendCheckTls {
    let provider = tls.provider.as_ref().map(|provider| match provider {
        TlsProvider::Rustls => "rustls",
        TlsProvider::NativeTls => "native-tls",
    });

    let crypto = tls.rustls.crypto.as_ref().map(|crypto| match crypto {
        RustlsCrypto::Aws => "aws",
        RustlsCrypto::Ring => "ring",
    });

    BackendCheckTls {
        provider: provider.map(String::from),
        crypto: crypto.map(String::from),
        cert: tls.server.as_ref().map(|s| s.cert.display().to_string()),
        client_cert: tls.client.as_ref().map(|c| c.cert.display().to_string()),
    }
}

/// The row named `name`, coded as the failure object of any other command
/// would be, a code read off a typed error included.
fn row(name: &'static str, result: Result<()>) -> BackendCheck<ErrorCode> {
    BackendCheck::new(name, result.map_err(|err| with_code(err, error::code)))
}
