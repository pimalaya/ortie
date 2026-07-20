# Provider notes

Per-provider OAuth 2.0 quirks learned the hard way while testing the discovery
wizard. The wizard already applies these; this is the reference for manual setups
and for understanding why a hand-written config might fail. The full field
reference lives in `config.sample.toml`; the layering rationale (why some of this
is an ortie-local stopgap rather than discovery-driven) lives in
`discovery-layering.md`.

## Google

- Endpoints: use `https://accounts.google.com/o/oauth2/v2/auth` and
  `https://oauth2.googleapis.com/token`. The legacy pair (`o/oauth2/auth` and
  `https://www.googleapis.com/oauth2/v3/token`) can be rejected at the consent
  screen with "This app is blocked". The wizard fills the current endpoints; only
  a hand-written config copied from old docs hits this.
- No dynamic registration: Google publishes no `registration_endpoint`, so the
  wizard offers the public Thunderbird application or a custom entry, never RFC
  7591.
- Sensitive scopes: Gmail (`https://mail.google.com`) and CardDAV are sensitive.
  The Thunderbird public client is verified and works; an unverified *own*
  application only works for Google accounts added as test users on its OAuth
  consent screen, otherwise it is blocked.
- Refresh token: Google uses `extras.access_type = "offline"`, not the
  `offline_access` scope.

## Microsoft

- No dynamic registration: like Google, Microsoft publishes no
  `registration_endpoint`; the wizard offers Thunderbird or a custom entry.
- IMAP/POP/SMTP scopes use the `https://outlook.office.com/` resource
  (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), NOT
  `https://outlook.office365.com/`. The `outlook.office365.com` / `smtp.office365.com`
  names are the mail server hosts, not scope resources; using them as the scope
  resource fails with `invalid_scope` ("The provided resource value for the input
  parameter 'scope' is not valid").
- One token for several services: the wizard groups the IMAP, POP and SMTP grants
  (same endpoints, different scope) into a single choice whose scope is their
  union, so one token covers all of them. Trim what you do not need in the scope
  multi-select.
- Redirect: the Thunderbird application is registered with `https://localhost`
  (port 443), which the local listener cannot bind unprivileged, so `auth get`
  falls back to the manual `auth resume`. A registered own application with an
  `http://127.0.0.1` redirect captures automatically.
- Graph is separate: the Thunderbird application is registered for Outlook
  IMAP/SMTP, not the Graph API. Graph tokens need a Graph-registered client and
  `https://graph.microsoft.com/*` scopes (see the Microsoft Graph recipe in
  `config.sample.toml`). Personal Microsoft accounts may receive an opaque token
  the Graph API rejects with `InvalidAuthenticationToken`.
- Device grant: `grant = "device"` with `…/oauth2/v2.0/devicecode` and matching `…/token`.

## Fastmail

- Dynamic registration works, but Fastmail's registration endpoint accepts ONLY a
  reverse-DNS private-use redirect scheme (`org.pimalaya.ortie://redirect`); it
  refuses every http and loopback redirect with `invalid_redirect_uri`
  ("redirect_uri not valid scheme"). See `discovery-layering.md`.
- RFC 8707 resource is mandatory: the authorize endpoint bounces the request
  pre-consent (a straight redirect to "You can close this window now", no login or
  scope screen) unless `extras.resource = "https://api.fastmail.com/jmap/session"`
  is present. The value is the RFC 9728 resource identifier of the JMAP session.
- Scopes: the discovered grant carries none, and an empty scope bounces the
  authorize the same way a missing resource does. The wizard fills, and selects by
  default, the full advertised set
  (`urn:ietf:params:oauth:scope:{mail,contacts,calendars}` plus `offline_access`),
  since Fastmail cannot complete on a desktop anyway; trim in the multi-select.
- Desktop limitation: even a valid request redirects to the private-use scheme,
  which a desktop browser cannot hand back to ortie, so the code is stranded
  behind "You can close this window now". There is nothing to do about the custom
  scheme on a desktop. To actually capture a token, pick the pre-registered
  Thunderbird application (loopback redirect) instead of dynamic registration, or
  register a system handler for the scheme. On mobile the OS routes the scheme
  back to the app, which is where private-use schemes belong.
