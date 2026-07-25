---
cairn: spec
capability: provider-quirks
status: current
---

# Provider quirks

Per-provider OAuth 2.0 quirks learned while testing the wizard. The wizard already applies these; this is the reference for manual setups and for understanding why a hand-written config might fail. The full field reference lives in `config.sample.toml`.

## Google

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`, so the wizard offers the Thunderbird public application or a custom entry. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope.

## Microsoft

### Requirement: Outlook scope resource
IMAP/POP/SMTP scopes SHALL use the `https://outlook.office.com/` resource (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), never `outlook.office365.com` (a mail host, not a scope resource, rejected with `invalid_scope`). The wizard groups the IMAP, POP and SMTP grants into one choice whose scope is their union. Microsoft publishes no `registration_endpoint`. The Thunderbird application is registered with `https://localhost` (port 443), which the unprivileged listener cannot bind, so `auth get` falls back to manual `auth resume`. Graph needs a Graph-registered client and `https://graph.microsoft.com/*` scopes. The device grant uses `.../oauth2/v2.0/devicecode` with the matching `.../token`.

## Fastmail

### Requirement: Fastmail resource and redirect
The Fastmail authorize endpoint SHALL receive `extras.resource = "https://api.fastmail.com/jmap/session"` (RFC 8707, the RFC 9728 resource identifier of the JMAP session) and a non-empty scope, or it bounces pre-consent. Dynamic registration accepts only a reverse-DNS private-use redirect scheme (`org.pimalaya.ortie://redirect`), refusing every http/loopback redirect. A desktop browser cannot hand the private-use scheme back to Ortie, so completing on desktop requires the pre-registered Thunderbird application (loopback redirect) or a registered system handler; on mobile the OS routes the scheme back to the app.
