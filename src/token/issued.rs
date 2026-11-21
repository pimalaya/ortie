//! # Issued token receipt
//!
//! What a grant or a refresh answers once the new access token is stored:
//! the account it was stored for, when it expires and whether a refresh
//! token came with it. The token itself never appears, `token show` being
//! the command that prints it.

use std::fmt;

use pimalaya_cli::duration::friendly;
use schemars::JsonSchema;
use serde::Serialize;

use io_oauth::rfc6749::issue_access_token::Oauth20AccessTokenSuccessParams;

/// The access token a grant or a refresh just stored.
///
/// Printed by `token refresh`, `auth listen`, `auth resume`, and `auth get`
/// when its grant issues the token in-process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenIssuedOutput {
    /// The name of the account the token was stored for.
    pub account: String,
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
    /// The receipt of a token a grant issued for `account`.
    pub fn issued(account: &str, res: &Oauth20AccessTokenSuccessParams) -> Self {
        Self {
            account: account.to_owned(),
            expires_in: res.expires_in,
            with_refresh_token: res.refresh_token.is_some(),
            refreshed: false,
        }
    }

    /// The receipt of a token a refresh issued for `account`.
    pub fn refreshed(account: &str, res: &Oauth20AccessTokenSuccessParams) -> Self {
        Self {
            refreshed: true,
            ..Self::issued(account, res)
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
        let account = &self.account;
        write!(f, "Access token of account {account} successfully {verb}")?;

        match self.expires_in {
            None => writeln!(f, " (unknown expiry)"),
            Some(exp) => {
                writeln!(f, " (expires in {})", friendly(exp.saturating_add(1)))
            }
        }
    }
}
