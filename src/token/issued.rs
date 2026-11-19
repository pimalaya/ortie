//! # Issued token receipt
//!
//! What a grant or a refresh answers once the new access token is stored:
//! when it expires and whether a refresh token came with it. The token
//! itself never appears, `token show` being the command that prints it.

use std::{fmt, time::Duration};

use jiff::{Span, SpanRound, Unit, fmt::friendly::SpanPrinter};
use schemars::JsonSchema;
use serde::Serialize;

use io_oauth::rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams;

/// The access token a grant or a refresh just stored.
///
/// Printed by `token refresh`, `auth resume`, and `auth get` when its grant
/// issues the token in-process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenIssuedOutput {
    /// The token's lifetime in seconds as the server stated it, `null`
    /// when it stated none.
    pub expires_in: Option<u64>,
    /// Whether a refresh token came with it.
    pub with_refresh_token: bool,
    /// Whether it replaces an expiring token rather than ending a grant,
    /// which only words the text form.
    #[serde(skip)]
    #[schemars(skip)]
    refreshed: bool,
}

impl TokenIssuedOutput {
    /// The receipt of a token a grant issued.
    pub fn issued(res: &Oauth20AccessTokenSuccessParams) -> Self {
        Self {
            expires_in: res.expires_in,
            with_refresh_token: res.refresh_token.is_some(),
            refreshed: false,
        }
    }

    /// The receipt of a token a refresh issued.
    pub fn refreshed(res: &Oauth20AccessTokenSuccessParams) -> Self {
        Self {
            refreshed: true,
            ..Self::issued(res)
        }
    }
}

impl fmt::Display for TokenIssuedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = if self.refreshed {
            "refreshed"
        } else {
            "issued"
        };
        write!(f, "Access token successfully {verb}")?;

        match self.expires_in {
            None => writeln!(f, " (unknown expiry)"),
            Some(exp) => {
                writeln!(f, " (expires in {})", friendly(exp.saturating_add(1)))
            }
        }
    }
}

/// Prints a duration in seconds in jiff's friendly form, balanced up to
/// days of 24 hours: `2d 4h 30m`, `0s` when empty.
///
/// A duration too long for a jiff span stops at hours.
pub fn friendly(secs: u64) -> String {
    let rounding = SpanRound::new().largest(Unit::Day).days_are_24_hours();

    let span = i64::try_from(secs)
        .ok()
        .and_then(|secs| Span::new().try_seconds(secs).ok())
        .and_then(|span| span.round(rounding).ok());

    match span {
        Some(span) => format!("{span:#}"),
        None => SpanPrinter::new().unsigned_duration_to_string(&Duration::from_secs(secs)),
    }
}

#[cfg(test)]
mod tests {
    use crate::token::issued::friendly;

    #[test]
    fn a_duration_prints_in_days_at_most() {
        let cases = [
            (0, "0s"),
            (90, "1m 30s"),
            (5400, "1h 30m"),
            (2 * 86_400 + 4 * 3600, "2d 4h"),
            (400 * 86_400 + 61, "400d 1m 1s"),
            (u64::MAX, "5124095576030431h 15s"),
        ];

        for (secs, text) in cases {
            assert_eq!(friendly(secs), text, "{secs}");
        }
    }
}
