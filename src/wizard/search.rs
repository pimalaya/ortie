//! Input-driven OAuth 2.0 grant discovery for the wizard.
//!
//! Mirrors the Himalaya wizard's search module, adapted to OAuth: the
//! input feeds io-pim-discovery's parallel discovery (fixed provider
//! rules, PACC, Mozilla autoconfig, RFC 6186 SRV, RFC 8620 JMAP
//! resolve, RFC 8414 authorization server metadata), and every OAuth
//! 2.0 flow it advertises becomes one selectable entry tagged with the
//! services sharing it. Grants of the same flow against the same
//! authorization server merge into one entry, whether they differ only
//! in scope (Microsoft's IMAP and SMTP, say) or in the spelling two
//! mechanisms gave the same endpoints, so a single token can cover
//! every service and the pick list holds one entry per real choice.
//!
//! An issuer never reaches the pick list as an issuer: it is resolved
//! through its RFC 8414 metadata into the grants it advertises, or
//! dropped.
//! The wizard configures only what it can discover, so there is no
//! hand-entry of endpoints anywhere here.

use std::{collections::BTreeSet, fmt, time::Duration};

use anyhow::{Context, Result};
use io_pim_discovery::{
    compose::{
        client::DiscoveryComposeClientStd,
        config::{DiscoveryAuthMethod, DiscoveryService, DiscoveryServiceConfig},
    },
    rfc8414::DiscoveryOauthServerMetadata,
    shared::dns::system_resolver,
};
use log::debug;
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

/// Fallback DNS resolver when the system one cannot be determined:
/// Cloudflare's `1.1.1.1` over TCP.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Upper bound on the parallel discovery fan-out. An unreachable
/// endpoint (a firewalled port, a black-hole host) must not stall the
/// interactive wizard, so mechanisms that have not reported by then
/// are abandoned and only what completed in time is offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(6);

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
        // NOTE: the flow is what the user arbitrates; the endpoints
        // behind it are one server's, since grants reduce by issuer.
        let flow = match &self.method {
            DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. } => {
                "OAuth 2.0 authorization code grant"
            }
            DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { .. } => {
                "OAuth 2.0 device authorization grant"
            }
            // NOTE: search() resolves every issuer into one of the two
            // grants above, so nothing else reaches the pick list.
            _ => return Ok(()),
        };

        let services = self
            .services
            .iter()
            .map(|service| service_name(*service))
            .collect::<Vec<_>>()
            .join(", ");

        // A grant resolved from a typed issuer URL carries no service:
        // the user named a server, not an address.
        if services.is_empty() {
            return write!(f, "{flow}");
        }

        write!(f, "{flow} ({services})")
    }
}

/// Searches every OAuth 2.0 grant reachable from `email` (a full
/// address or the synthesized `@domain` form) and returns one entry
/// per deduplicated grant.
///
/// The fan-out is bounded by [`DISCOVERY_TIMEOUT`], so an unreachable
/// mechanism costs the wizard a few seconds rather than the whole
/// prompt. An empty result means the caller stops.
pub fn search(email: &str) -> Result<Vec<Discovered>> {
    let client = compose_client();

    // NOTE: the OAuth-capable PIM services; POP3, WebDAV and
    // ManageSieve never advertise an OAuth flow of their own.
    let services = BTreeSet::from([
        DiscoveryService::Imap,
        DiscoveryService::Smtp,
        DiscoveryService::Jmap,
        DiscoveryService::Caldav,
        DiscoveryService::Carddav,
    ]);

    debug!("compose OAuth 2.0 services for {email}");
    let configs = client.compose_all_within(email, services, DISCOVERY_TIMEOUT)?;

    let mut found = collect_oauth(&configs);
    resolve_issuers(&client, &mut found);

    Ok(found)
}

/// Resolves a typed issuer URL into the single grant its RFC 8414
/// metadata advertises, or nothing when the metadata is unreachable or
/// names no usable endpoint.
///
/// This is the issuer twin of [`search`]: the user named an
/// authorization server directly instead of an address, so there is no
/// domain to fan out over and no service to tag the grant with.
pub fn search_issuer(input: &str) -> Result<Vec<Discovered>> {
    let issuer: Url = input
        .parse()
        .with_context(|| format!("Invalid issuer URL `{input}`"))?;

    debug!("resolve authorization server metadata for {issuer}");
    let Some(metadata) = compose_client().oauth_server(&issuer) else {
        return Ok(Vec::new());
    };

    let found = grants_of(&metadata)
        .into_iter()
        .map(|method| Discovered {
            method,
            services: BTreeSet::new(),
        })
        .collect();

    Ok(found)
}

/// Fetches the authorization server metadata backing `endpoints`,
/// guessing the issuer from each endpoint host (`https://<host>`) and
/// keeping the first document that answers.
///
/// No discovery mechanism carries this document alongside a composed
/// service config (the compose layer keeps flow endpoints only, and
/// the autoconfig sources never see server metadata), so the wizard
/// asks the provider itself, once per run: its `scopes_supported`
/// widens the scope options and its `registration_endpoint` decides
/// whether dynamic registration is offered.
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

/// TLS options for the wizard's HTTPS calls, pinned to HTTP/1.1: the
/// discovery mechanisms only ever speak it to `_well-known` endpoints.
pub fn wizard_tls() -> Tls {
    Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".to_string()],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The discovery client shared by the wizard's network steps, backed
/// by the system DNS resolver (with a public fallback), so the input
/// domain does not leak to a third-party resolver by default.
fn compose_client() -> DiscoveryComposeClientStd {
    let resolver = system_resolver().unwrap_or_else(|| {
        DEFAULT_RESOLVER
            .parse()
            .expect("default resolver must be a valid URL")
    });

    DiscoveryComposeClientStd::new(resolver, wizard_tls())
}

/// Collects the OAuth 2.0 methods across every discovered config,
/// grouped by flow and endpoints, each carrying the union of the
/// scopes and the set of services it authenticates.
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
/// An issuer alone configures no account: it names a server whose
/// endpoints are still unknown. Resolving it here keeps the pick list
/// made of entries the wizard can actually carry to a working config,
/// and folds a resolved grant into an identical one already found.
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

/// The grants an authorization server metadata document advertises:
/// the authorization code flow when it exposes an authorization
/// endpoint, the device flow when it exposes a device authorization
/// endpoint, both when it exposes both. RFC 8414 section 2 and RFC
/// 8628 section 4 let a server publish the two side by side, and a
/// machine with no browser wants the device flow even where a redirect
/// is possible, so neither hides the other: the pick list is where
/// that choice belongs. Both flows need the token endpoint, so a
/// document without one advertises nothing.
fn grants_of(metadata: &DiscoveryOauthServerMetadata) -> Vec<DiscoveryAuthMethod> {
    let Some(token_endpoint) = &metadata.token_endpoint else {
        return Vec::new();
    };

    let token_endpoint = token_endpoint.to_string();
    let scope =
        (!metadata.scopes_supported.is_empty()).then(|| metadata.scopes_supported.join(" "));

    let mut grants = Vec::new();

    if let Some(authorization_endpoint) = &metadata.authorization_endpoint {
        grants.push(DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint: authorization_endpoint.to_string(),
            token_endpoint: token_endpoint.clone(),
            scope: scope.clone(),
        });
    }

    if let Some(device_authorization_endpoint) = &metadata.device_authorization_endpoint {
        grants.push(DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
            device_authorization_endpoint: device_authorization_endpoint.to_string(),
            token_endpoint,
            scope,
        });
    }

    grants
}

/// Whether two grants are the same flow against the same authorization
/// server, ignoring their scope, so per-service grants merge into one.
///
/// The server is compared by the host of the endpoint that starts the
/// flow, not by the endpoint URLs: mechanisms disagree on the exact
/// spelling a provider writes its endpoints with (Mozilla's autoconfig
/// still carries Google's legacy `/o/oauth2/auth` where the fixed
/// provider rules carry `/o/oauth2/v2/auth`, both being the same
/// server), and the pick list must not ask the user to arbitrate
/// between two spellings of one thing. Compose yields its outputs in
/// mechanism-priority order, so the spelling kept is the most
/// authoritative one. Two genuinely different servers still differ.
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
/// parse, so two unparsable endpoints only ever match each other.
fn endpoint_host(endpoint: &str) -> String {
    Url::parse(endpoint)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default()
}

/// Unions the incoming grant's scope tokens into the existing grant's,
/// preserving order and dropping duplicates, so a merged grant
/// requests every grouped service's scopes at once.
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

/// Whether an authentication method is one of the OAuth 2.0 flows.
fn is_oauth(method: &DiscoveryAuthMethod) -> bool {
    matches!(
        method,
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant { .. }
            | DiscoveryAuthMethod::OauthDeviceAuthorizationGrant { .. }
            | DiscoveryAuthMethod::OauthIssuer(_)
    )
}

/// Lowercase wire name of a service, for the pick-list labels.
fn service_name(service: DiscoveryService) -> &'static str {
    match service {
        DiscoveryService::Imap => "imap",
        DiscoveryService::Pop3 => "pop3",
        DiscoveryService::Smtp => "smtp",
        DiscoveryService::Jmap => "jmap",
        DiscoveryService::Caldav => "caldav",
        DiscoveryService::Carddav => "carddav",
        DiscoveryService::Webdav => "webdav",
        DiscoveryService::Managesieve => "managesieve",
    }
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
        }
    }

    fn config(service: DiscoveryService, auth: Vec<DiscoveryAuthMethod>) -> DiscoveryServiceConfig {
        DiscoveryServiceConfig {
            service,
            endpoint: DiscoveryEndpoint::Http("https://example.test".to_string()),
            username: None,
            auth,
            source: DiscoveryConfigSource::Pacc,
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

    #[test]
    fn two_spellings_of_one_server_merge_into_the_first() {
        // What Google looks like once the fixed provider rules and
        // Mozilla's autoconfig have both described its mail grant: one
        // authorization server, two endpoint spellings, the provider
        // rules first since compose yields mechanisms in priority
        // order.
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
        } = &found[0].method
        else {
            panic!("expected an authorization code grant");
        };

        // The legacy spelling loses, and its services and scopes still
        // land in the entry that won.
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

    #[test]
    fn metadata_resolves_to_every_flow_it_advertises() {
        let scopes = ["mail", "offline_access"];

        // Both endpoints: both flows, the redirect one leading since a
        // desktop completes it without a second device.
        let both = metadata_of(
            Some("https://as/auth"),
            Some("https://as/device"),
            Some("https://as/token"),
            &scopes,
        );
        assert_eq!(
            grants_of(&both),
            [
                code_grant(
                    "https://as/auth",
                    "https://as/token",
                    Some("mail offline_access")
                ),
                DiscoveryAuthMethod::OauthDeviceAuthorizationGrant {
                    device_authorization_endpoint: "https://as/device".to_string(),
                    token_endpoint: "https://as/token".to_string(),
                    scope: Some("mail offline_access".to_string()),
                },
            ]
        );

        // Device only: no authorization endpoint to redirect to.
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
            }]
        );
    }

    #[test]
    fn metadata_without_a_runnable_flow_resolves_to_nothing() {
        // Neither endpoint: nothing the wizard can run.
        let bare = metadata_of(None, None, Some("https://as/token"), &[]);
        assert!(grants_of(&bare).is_empty());

        // No token endpoint: no flow can complete.
        let tokenless = metadata_of(Some("https://as/auth"), None, None, &[]);
        assert!(grants_of(&tokenless).is_empty());
    }
}
