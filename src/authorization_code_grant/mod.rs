//! Module dedicated to the section 4.1: Authorization Code Grant.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1>

pub mod access_token_request;
pub mod authorization_request;
pub mod authorization_response;
#[cfg(feature = "pkce")]
pub mod pkce;
pub mod state;
