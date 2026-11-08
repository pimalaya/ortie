//! # Grant discovery
//!
//! The input-driven OAuth 2.0 grant discovery of the wizard, mirroring
//! the Himalaya wizard's search module.
//!
//! The input feeds io-pim-discovery's parallel discovery (fixed
//! provider rules, PACC, Mozilla autoconfig, RFC 6186 SRV, RFC 8620
//! JMAP resolve, RFC 8414 server metadata), and every flow it
//! advertises becomes one entry tagged with the services sharing it.
//!
//! Grants of the same flow against the same authorization server merge
//! into one, whether they differ in scope or in the spelling two
//! mechanisms gave the same endpoints, so one token covers every
//! service and the pick list holds one entry per real choice.
//!
//! An issuer never reaches the pick list as an issuer: it resolves
//! through its RFC 8414 metadata into the grants it advertises, or it
//! is dropped. Nothing here is ever entered by hand.

use std::{collections::BTreeSet, fmt, time::Duration};

use anyhow::{Context, Result};
use clap::builder::{PossibleValuesParser, TypedValueParser};
use io_pim_discovery::{
    compose::{
        client::DiscoveryComposeClientStd,
        config::{DiscoveryAuthMethod, DiscoveryService, DiscoveryServiceConfig},
    },
    rfc8414::DiscoveryOauthServerMetadata,
    rfc9728::DiscoveryOauthResourceMetadata,
    shared::dns::system_resolver,
};
use log::debug;
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

/// Fallback DNS resolver when the system one cannot be determined:
/// Cloudflare's `1.1.1.1` over TCP.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Upper bound on the parallel discovery fan-out.
///
/// An unreachable endpoint must not stall the interactive wizard, so
/// mechanisms that have not reported by then are abandoned and only
/// what completed in time is offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(6);

/// The `grant_type` of the authorization code grant (RFC 6749 section
/// 4.1.3).
const AUTHORIZATION_CODE_GRANT_TYPE: &str = "authorization_code";

/// The `grant_type` of the device authorization grant (RFC 8628 section
/// 3.4).
const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// Every service a grant can be tagged with, in the order `--service`
/// lists them.
const SERVICES: [DiscoveryService; 14] = [
    DiscoveryService::Imap,
    DiscoveryService::Pop3,
    DiscoveryService::Smtp,
    DiscoveryService::Jmap,
    DiscoveryService::Caldav,
    DiscoveryService::Carddav,
    DiscoveryService::Webdav,
    DiscoveryService::Managesieve,
    DiscoveryService::Gmail,
    DiscoveryService::Gcal,
    DiscoveryService::Gpeople,
    DiscoveryService::Msgraph,
    DiscoveryService::MsgraphCalendar,
    DiscoveryService::MsgraphContacts,
];

/// One deduplicated OAuth 2.0 grant and the services sharing it.
///
/// The grant, not the service, is the unit of choice: a provider
/// advertising the same flow for IMAP, SMTP and CardDAV yields one
/// entry whose scopes are the union of all three.
#[derive(Debug, Eq, PartialEq)]
pub struct Discovered {
    /// The advertised OAuth 2.0 flow and its endpoints.
    pub method: DiscoveryAuthMethod,
    /// The PIM services this grant authenticates.
    pub services: BTreeSet<DiscoveryService>,
}

impl fmt::Display for Discovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: the flow is what the user arbitrates, the endpoints
        // behind it being one server's since grants reduce by issuer.
        let flow = match &self.method {
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. } => {
                "OAuth 2.0 authorization code grant"
            }
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { .. } => {
                "OAuth 2.0 device authorization grant"
            }
            // NOTE: search resolves every issuer into one of the grants
            // above, so nothing else reaches the pick list.
            _ => return Ok(()),
        };

        let services = self
            .services
            .iter()
            .map(|service| service_name(*service))
            .collect::<Vec<_>>()
            .join(", ");

        // NOTE: a grant resolved from a typed issuer URL carries no
        // service, the user having named a server, not an address.
        if services.is_empty() {
            return write!(f, "{flow}");
        }

        write!(f, "{flow} ({services})")
    }
}

/// Searches every OAuth 2.0 grant reachable from `email`, a full
/// address or the synthesized `@domain` form, one entry per grant.
///
/// Only `services` are composed when any is named, so a grant carries
/// their scopes alone.
///
/// [`DISCOVERY_TIMEOUT`] bounds the fan-out, so an unreachable
/// mechanism costs a few seconds rather than the whole prompt. An empty
/// result means the caller stops.
pub fn search(email: &str, services: &[DiscoveryService]) -> Result<Vec<Discovered>> {
    let client = compose_client();

    // NOTE: unfiltered, the API services stay out: Microsoft's Graph
    // would merge into the Outlook grant, and Entra issues one resource.
    let services = match services {
        [] => BTreeSet::from([
            DiscoveryService::Imap,
            DiscoveryService::Smtp,
            DiscoveryService::Jmap,
            DiscoveryService::Caldav,
            DiscoveryService::Carddav,
        ]),
        services => services.iter().copied().collect(),
    };

    debug!("compose OAuth 2.0 services {services:?} for {email}");
    let configs = client.compose_all_within(email, services, DISCOVERY_TIMEOUT)?;

    let mut found = collect_oauth(&configs);
    resolve_issuers(&client, &mut found);

    Ok(found)
}

/// Resolves a typed URL into the grants its RFC 8414 metadata
/// advertises, or into nothing when it names no usable endpoint.
///
/// The URL twin of [`search`]: the user named a server directly, so
/// there is no domain to fan out over and no service to tag the grant
/// with. A URL that is no issuer may be a protected resource: its RFC
/// 9728 metadata then names the authorization servers, whose grants are
/// bound to it.
pub fn search_issuer(input: &str) -> Result<Vec<Discovered>> {
    let url: Url = input
        .parse()
        .with_context(|| format!("Invalid issuer URL `{input}`"))?;

    let client = compose_client();

    debug!("resolve authorization server metadata for {url}");
    let grants = match client.oauth_server(&url) {
        Some(metadata) => grants_of(&metadata),
        None => resource_grants(&client, &url),
    };

    let found = grants
        .into_iter()
        .map(|method| Discovered {
            method,
            services: BTreeSet::new(),
        })
        .collect();

    Ok(found)
}

/// The grants of the authorization servers a protected resource's RFC
/// 9728 metadata names, each bound to the resource and its scopes.
fn resource_grants(client: &DiscoveryComposeClientStd, url: &Url) -> Vec<DiscoveryAuthMethod> {
    debug!("resolve protected resource metadata for {url}");
    let Some(resource) = client.oauth_resource(url) else {
        return Vec::new();
    };

    let mut grants = Vec::new();

    for server in &resource.authorization_servers {
        let Some(metadata) = client.oauth_server(server) else {
            debug!("skip unresolvable authorization server {server}");
            continue;
        };

        for mut grant in grants_of(&metadata) {
            bind_resource(&mut grant, &resource);
            grants.push(grant);
        }
    }

    grants
}

/// Binds a grant to a protected resource, narrowing its scopes to the
/// resource's when it lists any.
fn bind_resource(grant: &mut DiscoveryAuthMethod, metadata: &DiscoveryOauthResourceMetadata) {
    let (DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
        scope,
        resource,
        scopes_supported,
        ..
    }
    | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
        scope,
        resource,
        scopes_supported,
        ..
    }) = grant
    else {
        return;
    };

    *resource = Some(metadata.resource.to_string());

    if !metadata.scopes_supported.is_empty() {
        *scope = Some(metadata.scopes_supported.join(" "));
        *scopes_supported = metadata.scopes_supported.clone();
    }
}

/// Fetches the authorization server metadata backing `endpoints`,
/// keeping the first document an endpoint host answers with.
///
/// No discovery mechanism carries this document alongside a composed
/// service config, the compose layer keeping flow endpoints only, so
/// the wizard asks the provider itself, once per run.
pub fn metadata(hosts: &BTreeSet<String>) -> Option<DiscoveryOauthServerMetadata> {
    let client = compose_client();

    for host in hosts {
        let Ok(issuer) = format!("https://{host}").parse::<Url>() else {
            continue;
        };

        if let Some(metadata) = client.oauth_server(&issuer) {
            return Some(metadata);
        }
    }

    None
}

/// TLS options for the wizard's HTTPS calls, pinned to HTTP/1.1, the
/// only protocol its discovery mechanisms speak.
pub fn wizard_tls() -> Tls {
    Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".to_string()],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Parses a `--service` value, one of the wire names the pick list
/// labels the services with, an unknown one listing the valid ones.
pub fn service_parser() -> impl TypedValueParser<Value = DiscoveryService> {
    PossibleValuesParser::new(SERVICES.map(service_name)).map(|name| {
        SERVICES
            .into_iter()
            .find(|service| service_name(*service) == name)
            .expect("a possible value names a service")
    })
}

/// Keeps the grants tagged with at least one of `services`, every grant
/// when none is named.
///
/// A grant resolved from a typed URL carries no service, the URL having
/// named what it serves, so it is kept.
pub fn retain_services(found: &mut Vec<Discovered>, services: &[DiscoveryService]) {
    if services.is_empty() {
        return;
    }

    found.retain(|grant| {
        grant.services.is_empty()
            || services
                .iter()
                .any(|service| grant.services.contains(service))
    });
}

/// Wire name of a service, for the pick-list labels and `--service`.
pub fn service_name(service: DiscoveryService) -> &'static str {
    match service {
        DiscoveryService::Imap => "imap",
        DiscoveryService::Pop3 => "pop3",
        DiscoveryService::Smtp => "smtp",
        DiscoveryService::Jmap => "jmap",
        DiscoveryService::Caldav => "caldav",
        DiscoveryService::Carddav => "carddav",
        DiscoveryService::Webdav => "webdav",
        DiscoveryService::Managesieve => "managesieve",
        DiscoveryService::Gmail => "gmail",
        DiscoveryService::Gcal => "gcal",
        DiscoveryService::Gpeople => "gpeople",
        DiscoveryService::Msgraph => "msgraph",
        DiscoveryService::MsgraphCalendar => "msgraphCalendar",
        DiscoveryService::MsgraphContacts => "msgraphContacts",
    }
}

/// The discovery client shared by the wizard's network steps, backed by
/// the system DNS resolver so the input domain does not leak.
fn compose_client() -> DiscoveryComposeClientStd {
    let resolver = system_resolver().unwrap_or_else(|| {
        DEFAULT_RESOLVER
            .parse()
            .expect("default resolver must be a valid URL")
    });

    DiscoveryComposeClientStd::new(resolver, wizard_tls())
}

/// Collects the OAuth 2.0 methods across every discovered config,
/// grouped by flow and endpoints.
///
/// Each entry carries the union of the scopes and the set of services
/// it authenticates.
fn collect_oauth(configs: &[DiscoveryServiceConfig]) -> Vec<Discovered> {
    let mut discovered: Vec<Discovered> = Vec::new();

    for config in configs {
        for method in &config.auth {
            if !is_oauth(method) {
                continue;
            }

            match discovered
                .iter_mut()
                .find(|d| same_grant(&d.method, method))
            {
                Some(existing) => {
                    merge_scopes(&mut existing.method, method);
                    merge_resource(&mut existing.method, method);
                    existing.services.insert(config.service);
                }
                None => discovered.push(Discovered {
                    method: method.clone(),
                    services: BTreeSet::from([config.service]),
                }),
            }
        }
    }

    discovered
}

/// Turns every bare issuer entry into the concrete grants its RFC 8414
/// metadata advertises, dropping the ones that resolve to nothing.
///
/// An issuer alone configures no account, naming a server whose
/// endpoints are unknown, so resolving it here keeps the pick list made
/// of entries the wizard can carry to a working config.
fn resolve_issuers(client: &DiscoveryComposeClientStd, found: &mut Vec<Discovered>) {
    let issuers: Vec<usize> = found
        .iter()
        .enumerate()
        .filter(|(_, entry)| matches!(entry.method, DiscoveryAuthMethod::OauthIssuer(_)))
        .map(|(index, _)| index)
        .collect();

    // NOTE: walk backwards so the removals do not shift the indices
    // still to be visited.
    for index in issuers.into_iter().rev() {
        let entry = found.remove(index);

        let DiscoveryAuthMethod::OauthIssuer(issuer) = &entry.method else {
            continue;
        };

        let Ok(issuer) = issuer.parse::<Url>() else {
            debug!("drop unparsable issuer {issuer}");
            continue;
        };

        let resolved = client
            .oauth_server(&issuer)
            .map(|metadata| grants_of(&metadata))
            .unwrap_or_default();

        if resolved.is_empty() {
            debug!("drop issuer {issuer}, no usable endpoint advertised");
            continue;
        }

        for method in resolved {
            match found.iter_mut().find(|d| same_grant(&d.method, &method)) {
                Some(existing) => existing.services.extend(entry.services.iter().copied()),
                None => found.push(Discovered {
                    method,
                    services: entry.services.clone(),
                }),
            }
        }
    }
}

/// The grants an authorization server metadata document advertises, one
/// per endpoint that starts a flow.
///
/// RFC 8414 section 2 and RFC 8628 section 4 let a server publish both
/// side by side, and a machine with no browser wants the device flow
/// anyway, so neither hides the other: the pick list arbitrates.
///
/// Both flows need the token endpoint, so a document without one
/// advertises nothing.
///
/// A document listing `grant_types_supported` offers only the flows it
/// names; one omitting it defaults to `authorization_code` (RFC 8414
/// section 2), the device flow then following its endpoint alone.
fn grants_of(metadata: &DiscoveryOauthServerMetadata) -> Vec<DiscoveryAuthMethod> {
    let Some(token_endpoint) = &metadata.token_endpoint else {
        return Vec::new();
    };

    let token_endpoint = token_endpoint.to_string();
    let scope =
        (!metadata.scopes_supported.is_empty()).then(|| metadata.scopes_supported.join(" "));

    let mut grants = Vec::new();

    let grant_types = &metadata.grant_types_supported;
    let supports = |grant_type: &str| {
        grant_types.is_empty() || grant_types.iter().any(|supported| supported == grant_type)
    };

    if let Some(authorization_endpoint) = &metadata.authorization_endpoint
        && supports(AUTHORIZATION_CODE_GRANT_TYPE)
    {
        grants.push(DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint: authorization_endpoint.to_string(),
            token_endpoint: token_endpoint.clone(),
            scope: scope.clone(),
            resource: None,
            scopes_supported: metadata.scopes_supported.clone(),
        });
    }

    if let Some(device_authorization_endpoint) = &metadata.device_authorization_endpoint
        && supports(DEVICE_CODE_GRANT_TYPE)
    {
        grants.push(DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
            device_authorization_endpoint: device_authorization_endpoint.to_string(),
            token_endpoint,
            scope,
            resource: None,
            scopes_supported: metadata.scopes_supported.clone(),
        });
    }

    grants
}

/// Whether two grants are the same flow against the same authorization
/// server, ignoring their scope, so per-service grants merge into one.
///
/// The server is compared by the host of the endpoint that starts the
/// flow, not by the URLs: mechanisms disagree on the spelling, and the
/// pick list must not ask the user to arbitrate between two of them.
///
/// Compose yields its outputs in mechanism-priority order, so the
/// spelling kept is the most authoritative one. Two genuinely different
/// servers still differ.
fn same_grant(a: &DiscoveryAuthMethod, b: &DiscoveryAuthMethod) -> bool {
    match (a, b) {
        (
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: a_authorization,
                ..
            },
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: b_authorization,
                ..
            },
        ) => endpoint_host(a_authorization) == endpoint_host(b_authorization),
        (
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: a_device,
                ..
            },
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: b_device,
                ..
            },
        ) => endpoint_host(a_device) == endpoint_host(b_device),
        (DiscoveryAuthMethod::OauthIssuer(a), DiscoveryAuthMethod::OauthIssuer(b)) => a == b,
        _ => false,
    }
}

/// The lowercased host of an endpoint URL, empty when it does not
/// parse, so two unparsable endpoints match each other alone.
fn endpoint_host(endpoint: &str) -> String {
    Url::parse(endpoint)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default()
}

/// Unions the incoming grant's scope tokens into the existing grant's,
/// preserving order and dropping duplicates.
///
/// A merged grant therefore requests every grouped service's scopes at
/// once.
fn merge_scopes(existing: &mut DiscoveryAuthMethod, incoming: &DiscoveryAuthMethod) {
    let existing_scope = match existing {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { scope, .. }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { scope, .. } => scope,
        _ => return,
    };

    let incoming_scope = match incoming {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { scope, .. }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { scope, .. } => scope,
        _ => return,
    };

    let mut tokens: Vec<String> = existing_scope
        .as_deref()
        .map(|scope| scope.split_whitespace().map(ToString::to_string).collect())
        .unwrap_or_default();

    if let Some(incoming) = incoming_scope.as_deref() {
        for token in incoming.split_whitespace() {
            if !tokens.iter().any(|existing| existing == token) {
                tokens.push(token.to_string());
            }
        }
    }

    *existing_scope = (!tokens.is_empty()).then(|| tokens.join(" "));
}

/// Takes the incoming grant's resource, with the scopes it advertises,
/// when the existing grant has none.
///
/// One resource only: whether a server accepts several RFC 8707
/// indicators on one authorization request is unverified, so the first
/// service in compose priority order binds the token.
fn merge_resource(existing: &mut DiscoveryAuthMethod, incoming: &DiscoveryAuthMethod) {
    let (
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            resource,
            scopes_supported,
            ..
        }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
            resource,
            scopes_supported,
            ..
        },
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            resource: Some(incoming),
            scopes_supported: incoming_scopes,
            ..
        }
        | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
            resource: Some(incoming),
            scopes_supported: incoming_scopes,
            ..
        },
    ) = (existing, incoming)
    else {
        return;
    };

    if resource.is_none() {
        *resource = Some(incoming.clone());
        *scopes_supported = incoming_scopes.clone();
    }
}

/// Whether an authentication method is one of the OAuth 2.0 flows.
fn is_oauth(method: &DiscoveryAuthMethod) -> bool {
    matches!(
        method,
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. }
            | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { .. }
            | DiscoveryAuthMethod::OauthIssuer(_)
    )
}

#[cfg(test)]
mod tests {
    use io_pim_discovery::compose::config::{DiscoveryConfigSource, DiscoveryEndpoint};

    use super::*;

    fn code_grant(authorization: &str, token: &str, scope: Option<&str>) -> DiscoveryAuthMethod {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint: authorization.to_string(),
            token_endpoint: token.to_string(),
            scope: scope.map(ToString::to_string),
            resource: None,
            scopes_supported: Vec::new(),
        }
    }

    fn config(service: DiscoveryService, auth: Vec<DiscoveryAuthMethod>) -> DiscoveryServiceConfig {
        DiscoveryServiceConfig {
            service,
            endpoint: DiscoveryEndpoint::Http("https://example.test".to_string()),
            username: None,
            auth,
            source: DiscoveryConfigSource::Pacc,
            resolved: false,
        }
    }

    fn metadata_of(
        authorization: Option<&str>,
        device: Option<&str>,
        token: Option<&str>,
        scopes: &[&str],
    ) -> DiscoveryOauthServerMetadata {
        DiscoveryOauthServerMetadata {
            issuer: "https://as".parse().unwrap(),
            authorization_endpoint: authorization.map(|url| url.parse().unwrap()),
            token_endpoint: token.map(|url| url.parse().unwrap()),
            jwks_uri: None,
            registration_endpoint: None,
            scopes_supported: scopes.iter().map(ToString::to_string).collect(),
            response_types_supported: Vec::new(),
            response_modes_supported: Vec::new(),
            grant_types_supported: Vec::new(),
            token_endpoint_auth_methods_supported: Vec::new(),
            service_documentation: None,
            revocation_endpoint: None,
            introspection_endpoint: None,
            code_challenge_methods_supported: Vec::new(),
            device_authorization_endpoint: device.map(|url| url.parse().unwrap()),
        }
    }

    #[test]
    fn same_flow_across_services_merges_into_one_entry() {
        let imap = config(
            DiscoveryService::Imap,
            vec![code_grant(
                "https://as/auth",
                "https://as/token",
                Some("imap"),
            )],
        );
        let smtp = config(
            DiscoveryService::Smtp,
            vec![code_grant(
                "https://as/auth",
                "https://as/token",
                Some("smtp"),
            )],
        );

        let found = collect_oauth(&[imap, smtp]);
        assert_eq!(found.len(), 1);

        let DiscoveryAuthMethod::OauthAuthorizationCodeGrant { scope, .. } = &found[0].method
        else {
            panic!("expected an authorization code grant");
        };
        assert_eq!(scope.as_deref(), Some("imap smtp"));
        assert_eq!(found[0].services.len(), 2);
    }

    #[test]
    fn distinct_authorization_servers_stay_distinct_entries() {
        let jmap = config(
            DiscoveryService::Jmap,
            vec![code_grant("https://a/auth", "https://a/token", None)],
        );
        let caldav = config(
            DiscoveryService::Caldav,
            vec![code_grant("https://b/auth", "https://b/token", None)],
        );

        assert_eq!(collect_oauth(&[jmap, caldav]).len(), 2);
    }

    /// What Google looks like once the fixed provider rules and
    /// Mozilla's autoconfig have both described its mail grant.
    ///
    /// One server, two endpoint spellings, the provider rules first
    /// since compose yields mechanisms in priority order.
    #[test]
    fn two_spellings_of_one_server_merge_into_the_first() {
        let imap = config(
            DiscoveryService::Imap,
            vec![
                code_grant(
                    "https://accounts.google.com/o/oauth2/v2/auth",
                    "https://oauth2.googleapis.com/token",
                    Some("https://mail.google.com/"),
                ),
                code_grant(
                    "https://accounts.google.com/o/oauth2/auth",
                    "https://www.googleapis.com/oauth2/v3/token",
                    Some("https://mail.google.com/"),
                ),
            ],
        );
        let carddav = config(
            DiscoveryService::Carddav,
            vec![code_grant(
                "https://accounts.google.com/o/oauth2/v2/auth",
                "https://oauth2.googleapis.com/token",
                Some("https://www.googleapis.com/auth/carddav"),
            )],
        );

        let found = collect_oauth(&[imap, carddav]);
        assert_eq!(found.len(), 1);

        let DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint,
            token_endpoint,
            scope,
            ..
        } = &found[0].method
        else {
            panic!("expected an authorization code grant");
        };

        assert_eq!(
            authorization_endpoint,
            "https://accounts.google.com/o/oauth2/v2/auth"
        );
        assert_eq!(token_endpoint, "https://oauth2.googleapis.com/token");
        assert_eq!(
            scope.as_deref(),
            Some("https://mail.google.com/ https://www.googleapis.com/auth/carddav")
        );
        assert_eq!(found[0].services.len(), 2);
    }

    #[test]
    fn non_oauth_methods_are_ignored() {
        let basic = config(
            DiscoveryService::Imap,
            vec![DiscoveryAuthMethod::Password, DiscoveryAuthMethod::Bearer],
        );

        assert!(collect_oauth(&[basic]).is_empty());
    }

    /// IMAP comes first without a resource, JMAP brings the first one
    /// and CalDAV a second one, which does not replace it.
    #[test]
    fn merged_grant_takes_the_first_resource_with_its_scopes() {
        let bound =
            |resource: &str, scopes: &[&str]| DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                authorization_endpoint: "https://as/auth".to_string(),
                token_endpoint: "https://as/token".to_string(),
                scope: None,
                resource: Some(resource.to_string()),
                scopes_supported: scopes.iter().map(ToString::to_string).collect(),
            };

        let imap = config(
            DiscoveryService::Imap,
            vec![code_grant("https://as/auth", "https://as/token", None)],
        );
        let jmap = config(
            DiscoveryService::Jmap,
            vec![bound("https://api/jmap", &["mail", "offline_access"])],
        );
        let caldav = config(
            DiscoveryService::Caldav,
            vec![bound("https://dav/calendars", &["calendars"])],
        );

        let found = collect_oauth(&[imap, jmap, caldav]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].method,
            bound("https://api/jmap", &["mail", "offline_access"])
        );
    }

    /// Both endpoints yield both flows, the redirect one leading since
    /// a desktop completes it without a second device.
    #[test]
    fn metadata_resolves_to_every_flow_it_advertises() {
        let scopes = ["mail", "offline_access"];

        let both = metadata_of(
            Some("https://as/auth"),
            Some("https://as/device"),
            Some("https://as/token"),
            &scopes,
        );
        assert_eq!(
            grants_of(&both),
            [
                DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
                    authorization_endpoint: "https://as/auth".to_string(),
                    token_endpoint: "https://as/token".to_string(),
                    scope: Some("mail offline_access".to_string()),
                    resource: None,
                    scopes_supported: scopes.map(ToString::to_string).to_vec(),
                },
                DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                    device_authorization_endpoint: "https://as/device".to_string(),
                    token_endpoint: "https://as/token".to_string(),
                    scope: Some("mail offline_access".to_string()),
                    resource: None,
                    scopes_supported: scopes.map(ToString::to_string).to_vec(),
                },
            ]
        );

        let device = metadata_of(
            None,
            Some("https://as/device"),
            Some("https://as/token"),
            &[],
        );
        assert_eq!(
            grants_of(&device),
            [DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                device_authorization_endpoint: "https://as/device".to_string(),
                token_endpoint: "https://as/token".to_string(),
                scope: None,
                resource: None,
                scopes_supported: Vec::new(),
            }]
        );
    }

    /// Neither flow endpoint leaves nothing the wizard can run, and no
    /// token endpoint leaves nothing that could complete.
    /// A document naming its grant types offers only those, whatever
    /// endpoints it publishes.
    #[test]
    fn listed_grant_types_gate_the_offered_flows() {
        let mut metadata = metadata_of(
            Some("https://as/auth"),
            Some("https://as/device"),
            Some("https://as/token"),
            &[],
        );
        metadata.grant_types_supported = vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ];

        let grants = grants_of(&metadata);
        assert_eq!(grants.len(), 1);
        assert!(matches!(
            grants[0],
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. }
        ));
    }

    /// A resource narrows the server's scopes to its own and becomes
    /// the grant's resource.
    #[test]
    fn a_resource_binds_the_grants_of_its_servers() {
        let metadata = metadata_of(
            Some("https://as/auth"),
            None,
            Some("https://as/token"),
            &["openid", "mail"],
        );
        let resource: DiscoveryOauthResourceMetadata = serde_json::from_value(serde_json::json!({
            "resource": "https://api.example.test/jmap",
            "authorization_servers": ["https://as/"],
            "scopes_supported": ["mail"],
        }))
        .unwrap();

        let mut grant = grants_of(&metadata).remove(0);
        bind_resource(&mut grant, &resource);

        let DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            scope,
            resource,
            scopes_supported,
            ..
        } = grant
        else {
            panic!("expected an authorization code grant");
        };
        assert_eq!(resource.as_deref(), Some("https://api.example.test/jmap"));
        assert_eq!(scope.as_deref(), Some("mail"));
        assert_eq!(scopes_supported, ["mail"]);
    }

    /// A mail grant, a contacts grant on another server, and the untagged
    /// grant a typed URL resolves to.
    #[test]
    fn the_service_filter_keeps_the_grants_serving_a_named_service() {
        let tagged = |authorization: &str, services: &[DiscoveryService]| Discovered {
            method: code_grant(authorization, "https://as/token", None),
            services: services.iter().copied().collect(),
        };

        let found = || {
            vec![
                tagged(
                    "https://mail/auth",
                    &[DiscoveryService::Imap, DiscoveryService::Smtp],
                ),
                tagged("https://dav/auth", &[DiscoveryService::Carddav]),
                tagged("https://typed/auth", &[]),
            ]
        };

        let mut all = found();
        retain_services(&mut all, &[]);
        assert_eq!(all, found());

        let mut contacts = found();
        retain_services(&mut contacts, &[DiscoveryService::Carddav]);
        assert_eq!(contacts, [found().remove(1), found().remove(2)]);

        let mut any = found();
        retain_services(
            &mut any,
            &[DiscoveryService::Smtp, DiscoveryService::Caldav],
        );
        assert_eq!(any, [found().remove(0), found().remove(2)]);

        let mut none = found();
        retain_services(&mut none, &[DiscoveryService::Gmail]);
        assert_eq!(none, [found().remove(2)]);
    }

    #[test]
    fn metadata_without_a_runnable_flow_resolves_to_nothing() {
        let bare = metadata_of(None, None, Some("https://as/token"), &[]);
        assert!(grants_of(&bare).is_empty());

        let tokenless = metadata_of(Some("https://as/auth"), None, None, &[]);
        assert!(grants_of(&tokenless).is_empty());
    }
}
