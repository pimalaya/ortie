//! # Account check command
//!
//! The `account check` subcommand: whether the account's token storage
//! reads, whether the token it holds is usable, and whether each endpoint
//! its grant uses answers, one row each, the `https` endpoint rows naming
//! the TLS configured. Each row prints as an event once checked, then
//! the report lists them all.

use std::{
    fmt,
    io::{Read, Write},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use io_oauth::{
    client::Oauth20ClientStd, rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams,
};
use log::debug;
use pimalaya_cli::{error::PartialOutcome, fields::Fields, printer::Printer, sanitize};
use pimalaya_stream::tls::{RustlsCrypto, Tls, TlsProvider};
use schemars::JsonSchema;
use serde::Serialize;
use url::{Position, Url};

use crate::{
    account::{Account, TOKEN_MISSING},
    config::GrantConfig,
    error::{self, ErrorCode},
    token::{
        refresh::{RefreshAction, refresh_action},
        show::is_expired,
    },
};

/// The most bytes read looking for the status line of an answer.
const STATUS_LINE_MAX: usize = 1024;

/// Check that the account's token storage, token and endpoints work.
///
/// Prints one row per check, each OK or FAIL with why, every row checked
/// whatever the others did:
///
/// - `storage` runs `storage.read.command` and parses the token it
///   prints; the write command never runs, since it would replace the
///   stored token.
///
/// - `token` reads that token, sending nothing: it fails when none is
///   stored, and when it expired with no refresh token on a grant that
///   cannot run again by itself, `auth get` being the way out. An expired
///   token that auto-refresh or `token refresh` renews passes.
///
/// - `endpoints.<key>`, one row per endpoint the grant uses
///   (`authorization`, `device-authorization`, `token`, `revocation`):
///   one the grant needs and the account leaves out fails, a configured
///   one is sent a `HEAD` request carrying no credential, any HTTP answer
///   counting. The redirection is not checked.
///
/// None proves the provider accepts the client: only a grant does.
///
/// Each check prints an event as soon as it finishes, a sentence such as
/// `Reaching the token endpoint… OK` or `… FAIL (<why>)` on a terminal
/// (none when stdout is redirected), then a blank line and the report:
/// how many checks passed or failed, then each failed one and why.
///
/// Exits 0 when every row passes, 3 when one fails, the report being the
/// output rather than a failure object, and 1 on an account it cannot
/// take.
///
/// JSON output: one `{"event": "check-finished", "account", "check",
/// "ok", "code", "error", "tls"}` line per check as it finishes, then
/// `{"account", "checks": [{"check", "ok", "code", "error", "tls"}]}`,
/// `check` naming the row, `code` the stable code of its failure, `tls`
/// the TLS configured for an `https` endpoint, `null` on the other rows.
#[derive(Debug, Parser)]
pub struct AccountCheckCommand;

impl AccountCheckCommand {
    /// Checks the storage, the token, then the endpoints of `account`,
    /// printing every outcome whatever the others were.
    pub fn execute(self, printer: &mut impl Printer, account: &Account) -> Result<()> {
        let total = 2 + endpoints(account)
            .iter()
            .filter(|(_, url, needed)| url.is_some() || *needed)
            .count();

        let mut checks = Vec::new();
        let mut finish = |row: AccountCheckRow| -> Result<()> {
            printer.event(AccountCheckEvent::CheckFinished {
                account: account.name.clone(),
                check: row.check,
                ok: row.ok,
                code: row.code,
                error: row.error.clone(),
                tls: row.tls.clone(),
                last: checks.len() + 1 == total,
            })?;
            checks.push(row);
            Ok(())
        };

        let stored = account.read_storage();

        let token = match &stored {
            Ok(token) => check_token(account.grant, token),
            Err(err) if error::code(err) == Some(ErrorCode::TokenMissing) => {
                Err(error::coded(ErrorCode::TokenMissing, TOKEN_MISSING))
            }
            Err(_) => Err(anyhow!(
                "No access token to check, the storage check failed"
            )),
        };

        finish(AccountCheckRow::new("storage", stored.map(|_| ())))?;
        finish(AccountCheckRow::new("token", token))?;

        for (key, url, needed) in endpoints(account) {
            match url {
                Some(url) => {
                    let reached =
                        reach(account, url).with_context(|| format!("Cannot reach {key} at {url}"));
                    let mut row = AccountCheckRow::new(key, reached);
                    row.tls = (url.scheme() == "https").then(|| tls_check(&account.tls));
                    finish(row)?;
                }
                None if needed => {
                    finish(AccountCheckRow::new(key, Err(error::endpoint_missing(key))))?;
                }
                None => (),
            }
        }

        let report = AccountCheckOutput {
            account: account.name.clone(),
            checks,
        };

        let ok = report.ok();
        printer.out(report)?;

        if !ok {
            return Err(PartialOutcome::default().into());
        }

        Ok(())
    }
}

/// Whether `token` is usable on `grant`: unexpired, or expired with a way
/// to renew it, a refresh token or a grant running again by itself.
fn check_token(grant: GrantConfig, token: &Oauth20AccessTokenSuccessParams) -> Result<()> {
    if !is_expired(token.issued_at, token.expires_in) {
        return Ok(());
    }

    match refresh_action(grant, token.refresh_token.is_some()) {
        RefreshAction::Refresh | RefreshAction::Reacquire => Ok(()),
        RefreshAction::Keep => {
            let message = "Expired access token with no refresh token, run `auth get` to issue one";
            Err(error::coded(ErrorCode::TokenExpired, message))
        }
    }
}

/// The endpoints the grant of `account` uses, in check order, each with
/// its configuration key, its URL when configured, and whether the grant
/// needs it.
fn endpoints(account: &Account) -> Vec<(&'static str, Option<&Url>, bool)> {
    let authorization = (
        "endpoints.authorization",
        account.authorization_endpoint.as_ref(),
        true,
    );
    let device_authorization = (
        "endpoints.device-authorization",
        account.device_authorization_endpoint.as_ref(),
        true,
    );
    let token = ("endpoints.token", account.token_endpoint.as_ref(), true);
    let revocation = (
        "endpoints.revocation",
        account.revocation_endpoint.as_ref(),
        false,
    );

    match account.grant {
        GrantConfig::AuthorizationCode => vec![authorization, token, revocation],
        GrantConfig::Device => vec![device_authorization, token, revocation],
        GrantConfig::ClientCredentials | GrantConfig::ClientCredentialsJwt => {
            vec![token, revocation]
        }
    }
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

/// The TLS of the endpoints as configured, `null` where the build's
/// default answers.
fn tls_check(tls: &Tls) -> AccountCheckTls {
    let provider = tls.provider.as_ref().map(|provider| match provider {
        TlsProvider::Rustls => "rustls",
        TlsProvider::NativeTls => "native-tls",
    });

    let crypto = tls.rustls.crypto.as_ref().map(|crypto| match crypto {
        RustlsCrypto::Aws => "aws",
        RustlsCrypto::Ring => "ring",
    });

    AccountCheckTls {
        provider: provider.map(String::from),
        crypto: crypto.map(String::from),
        cert: tls.server.as_ref().map(|s| s.cert.display().to_string()),
        client_cert: tls.client.as_ref().map(|c| c.cert.display().to_string()),
    }
}

/// The `account check` output, one row per check of the account.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCheckOutput {
    /// The account that was checked.
    pub account: String,
    /// The rows, `storage`, `token`, then one per endpoint the grant
    /// uses.
    pub checks: Vec<AccountCheckRow>,
}

impl AccountCheckOutput {
    /// Whether every row passed, the command exiting with a partial
    /// outcome otherwise.
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|check| check.ok)
    }
}

/// One sentence counting the checks that passed or failed, then each
/// failed check and why, aligned, the events having printed every row.
impl fmt::Display for AccountCheckOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let account = sanitize::text(&self.account);
        let total = self.checks.len();
        let failed: Vec<&AccountCheckRow> = self.checks.iter().filter(|c| !c.ok).collect();

        if failed.is_empty() {
            return writeln!(f, "All {total} checks of {account} passed");
        }

        writeln!(f, "{} of {total} checks of {account} failed:", failed.len())?;

        let mut fields = Fields::default();
        for check in failed {
            let why = check.error.as_deref().unwrap_or_default();
            fields.push(check.check, sanitize::text(why));
        }

        write!(f, "{fields}")
    }
}

/// What a check does, as its event sentence opens: `Reaching the token
/// endpoint` for `endpoints.token`.
fn action(check: &str) -> String {
    match check {
        "storage" => String::from("Reading the token storage"),
        "token" => String::from("Checking the stored token"),
        check => {
            let endpoint = check.strip_prefix("endpoints.").unwrap_or(check);
            format!("Reaching the {} endpoint", endpoint.replace('-', " "))
        }
    }
}

/// `OK` or `FAIL (<why>)`, then `, TLS` and the configured parts on an
/// `https` endpoint.
fn outcome(error: Option<&str>, tls: Option<&AccountCheckTls>) -> String {
    let mut outcome = match error {
        None => String::from("OK"),
        Some(err) => format!("FAIL ({})", sanitize::text(err)),
    };

    if let Some(tls) = tls {
        outcome.push_str(", TLS");

        let tls = tls.to_string();
        if !tls.is_empty() {
            outcome.push(' ');
            outcome.push_str(&tls);
        }
    }

    outcome
}

/// What `account check` prints while it runs, before its
/// [`AccountCheckOutput`], tagged by `event`.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(
    tag = "event",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum AccountCheckEvent {
    /// A check finished, printed as soon as it did: the row the result
    /// lists again under `checks`.
    CheckFinished {
        /// The account being checked.
        account: String,
        /// The check: `storage`, `token`, or the configuration key of an
        /// endpoint (`endpoints.token`).
        check: &'static str,
        /// Whether it passed.
        ok: bool,
        /// The stable code of the failure, `null` when it passed or the
        /// failure carries none.
        code: Option<ErrorCode>,
        /// Why it failed, `null` when it passed.
        error: Option<String>,
        /// The TLS configured for an `https` endpoint, `null` on the other
        /// rows.
        tls: Option<AccountCheckTls>,
        /// Whether no check follows, its text then ending on a blank line
        /// before the report.
        #[serde(skip)]
        #[schemars(skip)]
        last: bool,
    },
}

/// One sentence, `Reaching the token endpoint… OK` or `… FAIL (<why>)`,
/// the last one followed by a blank line.
impl fmt::Display for AccountCheckEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CheckFinished {
                check,
                error,
                tls,
                last,
                ..
            } => {
                let outcome = outcome(error.as_deref(), tls.as_ref());
                writeln!(f, "{}… {outcome}", action(check))?;

                if *last {
                    writeln!(f)?;
                }

                Ok(())
            }
        }
    }
}

/// The outcome of one check.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCheckRow {
    /// The check: `storage`, `token`, or the configuration key of an
    /// endpoint (`endpoints.token`).
    pub check: &'static str,
    /// Whether it passed.
    pub ok: bool,
    /// The stable code of the failure, `null` when it passed or the
    /// failure carries none.
    pub code: Option<ErrorCode>,
    /// Why it failed, `null` when it passed.
    pub error: Option<String>,
    /// The TLS configured for an `https` endpoint, `null` on the other
    /// rows.
    pub tls: Option<AccountCheckTls>,
}

impl AccountCheckRow {
    /// Folds the outcome of `check` into its row, coded as the failure
    /// object of any other command would be, `tls` left `null`.
    fn new(check: &'static str, result: Result<()>) -> Self {
        match result {
            Ok(()) => Self {
                check,
                ok: true,
                code: None,
                error: None,
                tls: None,
            },
            Err(err) => Self {
                check,
                ok: false,
                code: error::code(&err),
                error: Some(format!("{err:#}")),
                tls: None,
            },
        }
    }
}

/// The TLS an `https` endpoint is configured with, as the `tls` table
/// resolves it: `null` where the build's default answers.
#[derive(Clone, Debug, Default, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCheckTls {
    /// The TLS implementation, `rustls` or `native-tls`; `null` for the
    /// build's first.
    pub provider: Option<String>,
    /// The cryptographic provider of rustls, `aws` or `ring`; `null` for
    /// the build's first.
    pub crypto: Option<String>,
    /// The PEM certificate trusted beside the platform's; `null` for the
    /// platform's alone.
    pub cert: Option<String>,
    /// The PEM client certificate presented; `null` for none.
    pub client_cert: Option<String>,
}

/// The configured parts, `native-tls, cert /etc/ca.pem`, empty when the
/// build's defaults answer every one.
impl fmt::Display for AccountCheckTls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = [
            self.provider.clone(),
            self.crypto.clone(),
            self.cert.as_ref().map(|cert| format!("cert {cert}")),
            self.client_cert
                .as_ref()
                .map(|cert| format!("client cert {cert}")),
        ];

        let parts: Vec<String> = parts.into_iter().flatten().collect();
        write!(f, "{}", sanitize::text(&parts.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use crate::account::check::{AccountCheckEvent, AccountCheckTls};

    /// An event prints as one sentence naming what was checked, its error
    /// defused, the last one followed by a blank line.
    #[test]
    fn an_event_prints_one_sentence() {
        let event = |check, error: Option<&str>, tls: Option<AccountCheckTls>, last| {
            AccountCheckEvent::CheckFinished {
                account: String::from("work"),
                check,
                ok: error.is_none(),
                code: None,
                error: error.map(String::from),
                tls,
                last,
            }
            .to_string()
        };

        assert_eq!(
            event("storage", None, None, false),
            "Reading the token storage… OK\n"
        );
        assert_eq!(
            event("token", None, None, false),
            "Checking the stored token… OK\n"
        );

        let tls = AccountCheckTls {
            provider: Some(String::from("rustls")),
            cert: Some(String::from("/etc/ortie/ca.pem")),
            ..AccountCheckTls::default()
        };
        assert_eq!(
            event("endpoints.device-authorization", None, Some(tls), false),
            "Reaching the device authorization endpoint… OK, TLS rustls, cert /etc/ortie/ca.pem\n"
        );

        let error = Some("Cannot reach\nendpoints.token\u{202e}");
        assert_eq!(
            event("endpoints.token", error, None, true),
            "Reaching the token endpoint… FAIL (Cannot reach\u{fffd}endpoints.token\u{fffd})\n\n"
        );
    }
}
