//! Module dedicated to the section 4.1.1: Authorization Request.
//!
//! Refs: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.1>

use alloc::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    string::String,
};

use url::Url;

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
    pub scope: BTreeSet<Cow<'a, str>>,

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

    /// Extra query parameters appended to the authorization URL.
    ///
    /// Useful for provider-specific parameters not covered by the
    /// typed fields (e.g. Google's `access_type`, `prompt`,
    /// `login_hint`; Microsoft's `tenant`, `resource`). Entries here
    /// override any equally-named default written from the typed
    /// fields above; entries already present in the endpoint URL's
    /// query string override entries here.
    pub extras: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
}

impl AuthorizationRequestParams<'_> {
    /// Build the authorization URL by writing the typed fields,
    /// applying `extras` on top, and preserving any pre-existing
    /// query parameters already present in `endpoint` (which take
    /// final precedence).
    // SAFETY: exposes the state and the PKCE code verifier
    pub fn build_url(&self, endpoint: &Url) -> Url {
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("response_type".into(), "code".into());
        params.insert("client_id".into(), self.client_id.as_ref().into());

        if let Some(state) = &self.state {
            params.insert(
                "state".into(),
                String::from_utf8_lossy(state.expose()).into_owned(),
            );
        }

        if let Some(uri) = &self.redirect_uri {
            params.insert("redirect_uri".into(), uri.as_ref().into());
        }

        if !self.scope.is_empty() {
            let mut scope = String::new();
            let mut glue = "";

            for token in &self.scope {
                scope.push_str(glue);
                scope.push_str(token);
                glue = " ";
            }

            params.insert("scope".into(), scope);
        }

        #[cfg(feature = "pkce")]
        if let Some(challenge) = &self.pkce_code_challenge {
            params.insert("code_challenge".into(), challenge.encode().into_owned());
            params.insert(
                "code_challenge_method".into(),
                challenge.method.as_str().into(),
            );
        }

        for (k, v) in &self.extras {
            params.insert(k.as_ref().into(), v.as_ref().into());
        }

        for (k, v) in endpoint.query_pairs() {
            params.insert(k.into_owned(), v.into_owned());
        }

        let mut url = endpoint.clone();
        let mut qm = url.query_pairs_mut();
        qm.clear();
        qm.extend_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        drop(qm);
        url
    }
}
