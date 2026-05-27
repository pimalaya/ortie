use std::{borrow::Cow, collections::HashSet};

use url::form_urlencoded::Serializer;

#[cfg(feature = "pkce")]
use crate::authorization_code_grant::pkce::PkceCodeChallenge;
use crate::authorization_code_grant::state::State;

/// The authorization request parameters from the authorization code
/// grant.
///
/// The client constructs the request URI by adding the following
/// parameters to the query component of the authorization endpoint
/// URI using the "application/x-www-form-urlencoded" format, per
/// Appendix B.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.1>
pub struct AuthorizationRequestParams<'a> {
    /// The client identifier.
    ///
    /// The authorization server issues the registered client a client
    /// identifier -- a unique string representing the registration
    /// information provided by the client.  The client identifier is
    /// not a secret; it is exposed to the resource owner and MUST NOT
    /// be used alone for client authentication.  The client
    /// identifier is unique to the authorization server.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-2.2>
    pub client_id: Cow<'a, str>,

    /// The redirection endpoint URI.
    ///
    /// After completing its interaction with the resource owner, the
    /// authorization server directs the resource owner's user-agent
    /// back to the client.  The authorization server redirects the
    /// user-agent to the client's redirection endpoint previously
    /// established with the authorization server during the client
    /// registration process or when making the authorization request.
    ///
    /// The redirection endpoint URI MUST be an absolute URI as
    /// defined by RFC3986 Section 4.3.  The endpoint URI MAY include
    /// an "application/x-www-form-urlencoded" formatted (per Appendix
    /// B) query component (RFC3986 Section 3.4), which MUST be
    /// retained when adding additional query parameters.  The
    /// endpoint URI MUST NOT include a fragment component.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-3.1.2>
    pub redirect_uri: Option<Cow<'a, str>>,

    /// The scope of the access request.
    ///
    /// The authorization and token endpoints allow the client to
    /// specify the scope of the access request using the "scope"
    /// request parameter.  In turn, the authorization server uses the
    /// "scope" response parameter to inform the client of the scope
    /// of the access token issued.
    ///
    /// The value of the scope parameter is expressed as a list of
    /// space- delimited, case-sensitive strings.  The strings are
    /// defined by the authorization server.  If the value contains
    /// multiple space-delimited strings, their order does not matter,
    /// and each string adds an additional access range to the
    /// requested scope.
    ///
    /// ```bnf
    /// scope       = scope-token *( SP scope-token )
    /// scope-token = 1*( %x21 / %x23-5B / %x5D-7E )
    /// ```
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-3.3>
    // TODO: validate scope tokens?
    pub scope: HashSet<Cow<'a, str>>,

    /// An opaque value used by the client to maintain state between
    /// the request and callback.
    ///
    /// The authorization server includes this value when redirecting
    /// the user-agent back to the client.  The parameter SHOULD be
    /// used for preventing cross-site request forgery.
    ///
    /// Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-10.12>
    pub state: Option<Cow<'a, State>>,

    #[cfg(feature = "pkce")]
    pub pkce_code_challenge: Option<Cow<'a, PkceCodeChallenge>>,
}

impl<'a> AuthorizationRequestParams<'a> {
    // SAFETY: exposes the state and the PKCE code verifier
    pub fn to_form_url_encoded_serializer(&self) -> Serializer<'a, String> {
        let mut serializer = Serializer::new(String::new());

        serializer.append_pair("response_type", "code");
        serializer.append_pair("client_id", &self.client_id);

        if let Some(state) = &self.state {
            let state = String::from_utf8_lossy(state.expose());
            serializer.append_pair("state", &state);
        }

        if let Some(uri) = &self.redirect_uri {
            serializer.append_pair("redirect_uri", uri);
        }

        if !self.scope.is_empty() {
            let mut scope = String::new();
            let mut glue = "";

            for token in &self.scope {
                scope.push_str(glue);
                scope.push_str(token);
                glue = " ";
            }

            serializer.append_pair("scope", &scope);
        }

        #[cfg(feature = "pkce")]
        if let Some(challenge) = &self.pkce_code_challenge {
            serializer.append_pair("code_challenge", &challenge.encode());
            serializer.append_pair("code_challenge_method", challenge.method.as_str());
        }

        serializer
    }

    // SAFETY: exposes the state and the PKCE code verifier
    pub fn to_form_url_encoded_string(&self) -> String {
        self.to_form_url_encoded_serializer().finish()
    }
}
