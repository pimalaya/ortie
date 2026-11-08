---
cairn: delta
change: pimalaya-applications-lead
---

# Delta

## ADDED Requirements

### Requirement: Pimalaya's own applications
The wizard SHALL know Pimalaya's own registrations, each offered ahead of any other well-known application registered against the same authorization server:

- Google: the Desktop client of the "Pimalaya" project, its client secret shipped with it (an installed application's secret is public by nature), serving `imap`, `smtp`, `gmail`, `gcal` and `gpeople`, and registered for `https://mail.google.com/`, `https://www.googleapis.com/auth/calendar.events`, `https://www.googleapis.com/auth/calendar.calendarlist.readonly`, `https://www.googleapis.com/auth/tasks` and `https://www.googleapis.com/auth/contacts`. It does not serve `caldav` nor `carddav`, whose APIs the project leaves off.
- Microsoft: the Pimalaya Entra application, redirected to `http://localhost`, serving `imap`, `smtp`, `msgraph`, `msgraphCalendar` and `msgraphContacts`, and registered for `offline_access`, `https://outlook.office.com/IMAP.AccessAsUser.All`, `https://outlook.office.com/SMTP.Send`, `Mail.ReadWrite`, `Mail.Send`, `Calendars.ReadWrite`, `Tasks.ReadWrite` and `Contacts.ReadWrite`.

#### Scenario: A contacts tool hands Google over
- GIVEN a Gmail address
- WHEN `ortie configure <address> --service gpeople` reaches the application step
- THEN the pick list leads with Pimalaya's application
- AND its scope step selects `https://www.googleapis.com/auth/contacts` alone

#### Scenario: Google over CardDAV
- GIVEN a Gmail address
- WHEN `ortie configure <address> --service carddav` reaches the application step
- THEN Pimalaya's application is not offered and Thunderbird's leads

## MODIFIED Requirements

### Requirement: Grant-based service reduction
Discovery SHALL reduce services to deduplicated OAuth 2.0 grants tagged with the services sharing them, and present the grant as the unit of choice. Two grants SHALL be reduced into one when they are the same flow against the same authorization server, compared by the host of the endpoint that starts the flow rather than by the endpoint URLs: mechanisms disagree on the exact spelling a provider's endpoints are written with, and the pick list must not ask the user to arbitrate between two spellings of one server. The first spelling seen wins the entry's endpoints, which is the highest-priority mechanism's since compose yields its outputs in mechanism-priority order, and the scopes and services of both are merged into it. The entry SHALL be labelled by its flow and the services it authenticates, never by its endpoints.

`--service`, repeatable, SHALL restrict discovery to the named services, written as the pick list labels them: `imap`, `pop3`, `smtp`, `jmap`, `caldav`, `carddav`, `webdav`, `managesieve`, `gmail`, `gcal`, `gpeople`, `msgraph`, `msgraphCalendar` or `msgraphContacts`. Only the named services are composed, so a grant carries their scopes alone, and the grants tagged with none of them are dropped. An unknown name SHALL be a usage error listing the valid ones. A grant resolved from a typed URL carries no service, the URL having named what it serves, and SHALL be kept. Without `--service`, discovery composes `imap`, `smtp`, `jmap`, `caldav` and `carddav`, and every grant is kept.

#### Scenario: A contacts tool hands its setup over
- GIVEN an address whose discovery yields a grant for `imap` and `smtp` and another, on a different authorization server, for `carddav`
- WHEN `ortie configure <address> --service carddav` runs
- THEN the `carddav` grant is carried through without a pick list

#### Scenario: A mail tool hands Microsoft over
- GIVEN a Microsoft address, whose IMAP, SMTP and Graph services share one authorization server
- WHEN `ortie configure <address> --service imap --service smtp` runs
- THEN the grant carries the Outlook IMAP and SMTP scopes and no Graph scope

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the authorization server metadata advertises a `registration_endpoint`, well-known public applications, then a custom entry. A well-known application SHALL be offered only when it is registered against the same authorization server as the grant and, for a grant tagged with services, serves at least one of them; Pimalaya's own lead the others. The decision reads the metadata probed once for the run rather than probing again. The step SHALL run before the scope step, and hand it the scope options its outcome allows. A pick list holding the custom entry alone SHALL be skipped entirely, that entry asking nothing.

### Requirement: Scopes follow the application
The scope step SHALL run after the application step, because what a token may request is a property of the application carrying it. The application step decides where the options come from:

- a well-known public application offers exactly the scopes its registration is granted, with the discovered ones selected, so a scope its client id was never verified for cannot reach the authorization request; a discovered scope the registration is granted under narrower names selects those names instead (Pimalaya's Google application reads `https://www.googleapis.com/auth/calendar` as `calendar.events` and `calendar.calendarlist.readonly`);
- dynamic registration prompts from the authorization server's advertised scopes, widened by the discovered ones, from inside the registration branch, since the scopes travel in the registration request;
- a custom application prompts for nothing and leaves the discovered scopes in the fragment, since nothing exposes the scopes tied to a registration the user made by hand.

#### Scenario: Thunderbird on Google
- GIVEN a Gmail account and the Thunderbird public application
- WHEN the wizard reaches the scope step
- THEN the options are the three scopes Thunderbird is registered for (`https://mail.google.com/`, `https://www.googleapis.com/auth/carddav`, `https://www.googleapis.com/auth/calendar`)
- AND a People API scope, which that client id is not verified for, is not offered

#### Scenario: Pimalaya on Google Calendar
- GIVEN a Gmail address configured with `--service gcal` and Pimalaya's application
- WHEN the wizard reaches the scope step
- THEN `calendar.events` and `calendar.calendarlist.readonly` are selected

### Requirement: Registration honours print-only
Dynamic registration SHALL run at wizard time with `token_endpoint_auth_method` none, grant and response types from the discovered grant, the discovered scopes, and client name `Ortie`. The issued client id (and client secret as the config secret shape) land inside the printed fragment. A failed registration reports through its spinner and falls back to the remaining pick-list entries.

#### Scenario: Provider without registration
- GIVEN Google or Microsoft, which publish no `registration_endpoint`
- WHEN the wizard reaches the application step
- THEN the pick list leads with Pimalaya's application, then Thunderbird's, not dynamic registration

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), open it, and capture the redirect on a loopback listener, then exchange the code, write storage, and fire the on-issue hooks.

### Requirement: Redirection resolution
When `endpoints.redirection` is set it SHALL be used verbatim, except an `http` loopback redirection naming no port, which Ortie binds on an ephemeral port of its host and sends with that port (RFC 8252 section 7.3). When it is not set, Ortie binds `127.0.0.1:0` and uses the resulting `http://127.0.0.1:<port>` URL. Both are exact-match loopback redirects under the permitted variable-port exception.

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`, so the wizard offers Pimalaya's application, Thunderbird's for IMAP, SMTP, CalDAV and CardDAV, or a custom entry. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope.

### Requirement: Outlook scope resource
IMAP/POP/SMTP scopes SHALL use the `https://outlook.office.com/` resource (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), never `outlook.office365.com` (a mail host, not a scope resource, rejected with `invalid_scope`). The wizard groups the IMAP, POP and SMTP grants into one choice whose scope is their union. Microsoft publishes no `registration_endpoint`. Pimalaya's application is registered with `http://localhost`, any port, and refuses `127.0.0.1`. The Thunderbird application is registered with `https://localhost` (port 443), which the unprivileged listener cannot bind, so `auth get` falls back to manual `auth resume`; it serves IMAP, POP and SMTP only. Graph needs a Graph-registered client, such as Pimalaya's, and Graph scopes, in a grant of its own since Entra issues a token for one resource. The device grant uses `.../oauth2/v2.0/devicecode` with the matching `.../token`.
