---
cairn: spec
capability: provider-quirks
status: current
---

# Provider quirks

Per-provider OAuth 2.0 quirks learned while testing against the providers. The provider sections of config.sample.toml apply them; this is the reference for an account written by hand and for understanding why one might fail. The full field reference lives in config.sample.toml.

## Google

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`: an account takes Pimalaya's application, registered for Gmail, IMAP and SMTP, the Calendar API (`calendar.events`, `calendar.calendarlist.readonly`), the People API and Tasks, not for CalDAV nor CardDAV, or an application of the user's own. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope. CardDAV and the People API take two scopes that are not interchangeable, `auth/carddav` and `auth/contacts`; CalDAV takes the whole `auth/calendar`. An application of the user's own is a Desktop client, whose refresh tokens expire after 7 days while its consent screen stays in testing.

## Microsoft

### Requirement: Outlook scope resource
IMAP/POP/SMTP scopes SHALL use the `https://outlook.office.com/` resource (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), never `outlook.office365.com` (a mail host, not a scope resource, rejected with `invalid_scope`). Microsoft publishes no `registration_endpoint`. Pimalaya's application is registered with `http://localhost`, any port, and refuses `127.0.0.1`. Graph needs a Graph-registered client, such as Pimalaya's, and Graph scopes, in an account of its own since Entra issues a token for one resource. The device grant uses `.../oauth2/v2.0/devicecode` with the matching `.../token`. An application of the user's own registers `http://localhost` under the "Mobile and desktop applications" platform, which an organisation refusing Pimalaya's application needs.

## Fastmail

### Requirement: Fastmail resource and redirect
The Fastmail authorize endpoint SHALL receive the JMAP session URL as RFC 8707 resource and a non-empty scope, or it bounces pre-consent; the session's RFC 9728 metadata names the resource. It forwards every authorization parameter to a strict check, so none beyond RFC 6749, PKCE, `resource` and `login_hint` is sent. Its dynamic registration answers the same client id to the same request and accepts only a reverse-DNS private-use redirect scheme (`org.pimalaya.ortie://redirect` in the sample), refusing every http/loopback redirect. A desktop browser cannot hand the private-use scheme back to Ortie, so `auth get` prints the manual `auth resume`; on mobile the OS routes the scheme back to the app.
