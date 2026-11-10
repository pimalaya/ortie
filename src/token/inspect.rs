//! # Token inspect command
//!
//! The `token inspect` subcommand, printing the metadata around the
//! stored access token rather than the token itself.

use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use humantime::{format_duration, format_rfc3339_seconds};
use pimalaya_cli::{fields::Fields, printer::Printer};
use schemars::JsonSchema;
use serde::{Serialize, Serializer};

use io_oauth::rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams;

use crate::account::Account;

/// Inspect metadata associated to the access token.
///
/// Unlike `token show`, this prints the token type, when it was issued,
/// when it expires, whether a refresh token came with it, and the
/// scopes it was granted. The token itself never appears.
#[derive(Debug, Parser)]
pub struct TokenInspectCommand;

impl TokenInspectCommand {
    /// Reads the token from storage and prints its metadata.
    pub fn execute(self, printer: &mut impl Printer, account: &mut Account) -> Result<()> {
        let response = account.resolve_token()?;
        printer.out(TokenInspectOutput::from(response))
    }
}

/// Printable metadata of the stored token, holding no secret.
///
/// It carries what the text rendering has always shown, so both
/// renderings agree on handing out nothing sensitive: the raw access
/// token comes from `token show`, and the refresh token from storage.
#[derive(Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInspectOutput {
    /// The token type the server issued, usually `Bearer`.
    pub token_type: String,
    /// When the token was issued, an RFC 3339 instant in UTC, `null` when
    /// unknown.
    #[serde(serialize_with = "serialize_rfc3339")]
    #[schemars(with = "Option<String>")]
    pub issued_at: Option<u64>,
    /// The lifetime granted at issuance, in seconds, when known.
    pub expires_in: Option<usize>,
    /// Whether a refresh token was stored alongside the access token.
    pub with_refresh_token: bool,
    /// The granted scope, when the server reported one.
    pub scope: Option<String>,
}

/// Serializes Unix epoch seconds as an RFC 3339 instant in UTC.
fn serialize_rfc3339<S: Serializer>(secs: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
    match secs {
        Some(secs) => {
            let instant = UNIX_EPOCH + Duration::from_secs(*secs);
            s.collect_str(&format_rfc3339_seconds(instant))
        }
        None => s.serialize_none(),
    }
}

impl From<Oauth20AccessTokenSuccessParams> for TokenInspectOutput {
    fn from(res: Oauth20AccessTokenSuccessParams) -> Self {
        Self {
            token_type: res.token_type,
            issued_at: res.issued_at,
            expires_in: res.expires_in,
            with_refresh_token: res.refresh_token.is_some(),
            scope: res.scope,
        }
    }
}

impl fmt::Display for TokenInspectOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        let mut fields = Fields::default();
        fields.push("Type", &self.token_type);

        if let (Some(issued_at), Some(now)) = (self.issued_at, now_epoch) {
            let elapsed = Duration::from_secs(now.saturating_sub(issued_at));
            fields.push("Issued", format!("{} ago", format_duration(elapsed)));
        }

        let expires = match self.expires_in {
            None => String::from("unknown"),
            Some(exp) => {
                let remaining = match (self.issued_at, now_epoch) {
                    (Some(issued_at), Some(now)) => (issued_at + exp as u64).saturating_sub(now),
                    _ => exp as u64,
                };

                match remaining {
                    0 => String::from("expired"),
                    secs => format!("in {}", format_duration(Duration::from_secs(secs))),
                }
            }
        };

        fields.push("Expires", expires);

        let refresh_token = if self.with_refresh_token { "yes" } else { "no" };
        fields.push("Refresh token", refresh_token);

        if let Some(scope) = &self.scope {
            let scopes: Vec<&str> = scope.split_whitespace().collect();
            fields.push("Scopes", scopes.join("\n"));
        }

        write!(f, "{fields}")
    }
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::*;

    #[test]
    fn output_serializes_no_secret() {
        let output = TokenInspectOutput::from(Oauth20AccessTokenSuccessParams {
            access_token: SecretString::from("access-secret"),
            token_type: String::from("Bearer"),
            expires_in: Some(3600),
            refresh_token: Some(SecretString::from("refresh-secret")),
            scope: Some(String::from("openid")),
            issued_at: Some(1_700_000_000),
        });

        let json = serde_json::to_string(&output).unwrap();

        assert!(!json.contains("access-secret"));
        assert!(!json.contains("refresh-secret"));
        assert!(json.contains("\"withRefreshToken\":true"));
        assert!(json.contains("\"tokenType\":\"Bearer\""));
        assert!(json.contains("\"issuedAt\":\"2023-11-14T22:13:20Z\""));
    }
}
