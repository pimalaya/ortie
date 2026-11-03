---
cairn: delta
change: discovery-layering
---

## ADDED Requirements

### Requirement: Resource indicators
An account MAY carry `resources`, a list of RFC 8707 resource indicator URLs, sent as one `resource` parameter each on the authorization request and on the code exchange.

### Requirement: Resource-bound grants
The wizard SHALL write the chosen grant's discovered resource into `resources`, and, when the grant requests no scope of its own, preselect the scopes its protected resource advertises. A grant merged across services SHALL keep the first resource seen in compose priority order, with that resource's scopes.

## MODIFIED Requirements

### Requirement: Extras passthrough
An account MAY carry a raw `[accounts.<name>.extras]` table whose keys are wire parameter names (never kebab-renamed) and whose values are strings. Extras are forwarded verbatim into the configured grant's initiation request (the authorization URL query for the authorization code grant). This carries provider options such as Google `access_type = "offline"` without Ortie learning provider-specific logic; an extra named `resource` replaces the `resources` list on the authorization URL.

### Requirement: Fastmail resource and redirect
The Fastmail authorize endpoint SHALL receive the JMAP session URL as RFC 8707 resource and a non-empty scope, or it bounces pre-consent; both come from the session's RFC 9728 metadata, without host matching. Dynamic registration accepts only a reverse-DNS private-use redirect scheme (`org.pimalaya.ortie://redirect`), refusing every http/loopback redirect. A desktop browser cannot hand the private-use scheme back to Ortie, so completing on desktop requires the pre-registered Thunderbird application (loopback redirect) or a registered system handler; on mobile the OS routes the scheme back to the app.

## REMOVED Requirements
