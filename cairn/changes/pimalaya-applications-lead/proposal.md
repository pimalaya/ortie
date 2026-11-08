---
cairn: change
id: pimalaya-applications-lead
status: landed
created: 2026-11-08
---

# Pimalaya's applications lead

## Why

Pimalaya now has its own OAuth 2.0 registrations: a Google Cloud project "Pimalaya" (an Android client, and since 2026-11-08 a Desktop client for the command-line interfaces), submitted for restricted-scope verification on 2026-11-08, and the Entra application `ba9b19e1-973e-4d7c-aed9-848bd2fee385` the Android app ships. The wizard offers only Thunderbird's public clients, so every Pimalaya CLI handing its OAuth 2.0 setup to Ortie signs in as Thunderbird, and Microsoft Graph has no client at all.

Reading the hand-off against the new registrations found three gaps that make a Pimalaya client unusable even once listed:

- `--service` filters the grants but not what discovery composes. `search` composes IMAP, SMTP, JMAP, CalDAV and CardDAV only, so `--service gmail`, `gcal`, `gpeople`, `msgraph`, `msgraphCalendar` or `msgraphContacts` (cardamum hands off `gpeople` and `msgraphContacts`, calendula `gcal` and `msgraphCalendar`) finds nothing and stops. And a kept grant carries the scopes of every service merged into it, so a mail hand-off on Google also asks for calendars and contacts.
- The Entra application accepts `http://localhost` on any port, and not `127.0.0.1` (probed on 2026-11-08: the consumers endpoint answers `invalid_request` for `http://127.0.0.1:53682` and the sign-in page for `http://localhost:53682`). A configured redirection is used verbatim, so `http://localhost` binds port 80, which an unprivileged process cannot, and the flow falls back to `auth resume`. Thunderbird's Google entry, pinned to `http://localhost`, has the same fallback today.
- Google discovery names `https://www.googleapis.com/auth/calendar`, which the Pimalaya project does not request: it asks `calendar.events` and `calendar.calendarlist.readonly` (narrowed on 2026-11-07 for the verification). With the Pimalaya client, a calendar grant would preselect no calendar scope.

## What

- **Pimalaya's applications in the known list, ahead of Thunderbird's.** Google: the Desktop client, its secret shipped (Google treats an installed app's secret as public), scopes `https://mail.google.com/`, `calendar.events`, `calendar.calendarlist.readonly`, `tasks`, `contacts`. Microsoft: the Entra application, redirection `http://localhost`, scopes `offline_access`, Outlook `IMAP.AccessAsUser.All` and `SMTP.Send`, Graph `Mail.ReadWrite`, `Mail.Send`, `Calendars.ReadWrite`, `Tasks.ReadWrite`, `Contacts.ReadWrite`. The secret is written in two pieces so secret scanners do not flag a value meant to be public.
- **A known application declares the services it serves**, and is offered only for a grant serving one of them. Pimalaya's Google client serves IMAP, SMTP and Google's APIs, not CalDAV or CardDAV (the project leaves those APIs off); Thunderbird's clients serve the protocols Thunderbird speaks. A grant resolved from a typed URL carries no service and is matched by its authorization server alone, as today.
- **A known application may be granted a discovered scope under narrower names.** Pimalaya's Google client reads `auth/calendar` as `calendar.events` and `calendar.calendarlist.readonly`.
- **`--service` composes the named services only**, so the API services become discoverable and the grant carries the scopes of what the caller named. Without `--service`, the composed set stays IMAP, SMTP, JMAP, CalDAV and CardDAV.
- **An `http` loopback redirection naming no port binds an ephemeral port** of its host and is sent with the bound port (RFC 8252 section 7.3), in `auth get` and in the MCP server.

## Not in this change

- The Google data disclosure before a sign-in with the Pimalaya client, which the Pimalaya project's verification requires of every interface (android cairn/changes/oauth-verification, §1): its own change, owed before the next release.
- A Pimalaya client at Fastmail (Thunderbird's was registered with Fastmail), to ask Fastmail for.
- Graph scopes requested alongside Outlook ones in one grant: Entra issues one resource per token. A run without `--service` on Microsoft composes IMAP and SMTP only, so it does not arise from the wizard.
