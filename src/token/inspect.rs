//! # Token inspect command
//!
//! The `token inspect` subcommand, printing the metadata around the
//! stored access token rather than the token itself.

use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use jiff::{SignedDuration, civil};
use pimalaya_cli::{duration::friendly, fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::{Serialize, Serializer, ser::Error as _};

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
    /// When the token expires, its issue time plus the lifetime the server
    /// granted, an RFC 3339 instant in UTC, in the past once expired, `null`
    /// when either is unknown.
    #[serde(serialize_with = "serialize_rfc3339")]
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<u64>,
    /// Whether a refresh token was stored alongside the access token.
    pub with_refresh_token: bool,
    /// The granted scope, when the server reported one.
    pub scope: Option<String>,
}

/// The last second an RFC 3339 instant can write, 9999-12-31T23:59:59Z.
const RFC3339_MAX_SECS: u64 = 253_402_300_799;

/// Serializes Unix epoch seconds as an RFC 3339 instant in UTC, an instant
/// past the year 9999 clamped to its last second.
fn serialize_rfc3339<S: Serializer>(secs: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
    match secs {
        Some(secs) => {
            let secs = (*secs).min(RFC3339_MAX_SECS) as i64;
            let epoch = civil::date(1970, 1, 1).at(0, 0, 0, 0);
            let instant = epoch
                .checked_add(SignedDuration::from_secs(secs))
                .map_err(S::Error::custom)?;

            s.collect_str(&format_args!("{instant}Z"))
        }
        None => s.serialize_none(),
    }
}

impl From<Oauth20AccessTokenSuccessParams> for TokenInspectOutput {
    fn from(res: Oauth20AccessTokenSuccessParams) -> Self {
        let expires_at = match (res.issued_at, res.expires_in) {
            (Some(issued_at), Some(lifetime)) => Some(issued_at.saturating_add(lifetime)),
            _ => None,
        };

        Self {
            token_type: res.token_type,
            issued_at: res.issued_at,
            expires_at,
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
        fields.push("Type", sanitize::text(&self.token_type));

        if let (Some(issued_at), Some(now)) = (self.issued_at, now_epoch) {
            let elapsed = friendly(now.saturating_sub(issued_at));
            fields.push("Issued", format!("{elapsed} ago"));
        }

        let expires = match (self.expires_at, now_epoch) {
            (Some(expires_at), Some(now)) if expires_at > now => {
                format!("in {}", friendly(expires_at - now))
            }
            (Some(_), Some(_)) => String::from("expired"),
            _ => String::from("unknown"),
        };

        fields.push("Expires", expires);

        let refresh_token = if self.with_refresh_token { "yes" } else { "no" };
        fields.push("Refresh token", refresh_token);

        if let Some(scope) = &self.scope {
            let scopes: Vec<_> = scope.split_whitespace().map(sanitize::text).collect();
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
        assert!(json.contains("\"expiresAt\":\"2023-11-14T23:13:20Z\""));
    }

    /// A lifetime the server states is not bounded, an instant is.
    #[test]
    fn a_lifetime_past_the_year_9999_expires_at_its_last_second() {
        let output = TokenInspectOutput::from(Oauth20AccessTokenSuccessParams {
            access_token: SecretString::from("at"),
            token_type: String::from("Bearer"),
            expires_in: Some(u64::MAX),
            refresh_token: None,
            scope: None,
            issued_at: Some(1_700_000_000),
        });

        let json = serde_json::to_value(&output).unwrap();

        assert_eq!(json["expiresAt"], "9999-12-31T23:59:59Z");
    }

    /// The type and the scopes are the server's words, printed defused.
    #[test]
    fn the_text_defuses_what_the_server_wrote() {
        let output = TokenInspectOutput::from(Oauth20AccessTokenSuccessParams {
            access_token: SecretString::from("at"),
            token_type: String::from("Bearer\x1b]52;c;cGF5bG9hZA==\x07"),
            expires_in: None,
            refresh_token: None,
            scope: Some(String::from("mail \u{202e}fdp.exe")),
            issued_at: None,
        });

        let text = output.to_string();

        assert!(!text.contains('\x1b'), "{text:?}");
        assert!(!text.contains('\u{202e}'), "{text:?}");
        assert!(text.contains("Scopes:         mail\n"), "{text:?}");
    }
}
