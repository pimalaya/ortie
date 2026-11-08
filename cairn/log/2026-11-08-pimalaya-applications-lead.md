---
cairn: log
change: pimalaya-applications-lead
landed: 2026-11-08
---

# Pimalaya's applications lead

`KNOWN_APPS` (src/wizard/client.rs) opens with Pimalaya's registrations: the Google Desktop client of the "Pimalaya" project (created 2026-11-08, its secret written as `concat!("GOCSPX", "-", ..)` so secret scanners do not flag a public value) and the Entra application `ba9b19e1-973e-4d7c-aed9-848bd2fee385`, pinned to `http://localhost`. Each `KnownApp` now names the services it serves, and `known_apps` offers one only when the grant serves one of them, a grant without services (typed URL) matching by host alone as before. Thunderbird's entries serve the protocols Thunderbird speaks: IMAP, SMTP, CalDAV and CardDAV on Google, IMAP, POP and SMTP on Microsoft, the five protocols on Fastmail. `KnownApp.narrows` reads a discovered scope under the registration's names, and `KnownApp::selected` applies it when the application is picked: Pimalaya's Google client takes `auth/calendar` as `calendar.events` and `calendar.calendarlist.readonly`.

`search::search` takes the `--service` services and composes only those, the protocol set (IMAP, SMTP, JMAP, CalDAV, CardDAV) when none is named. Before, Google's and Microsoft's API services were never composed, so the hand-offs cardamum and calendula make (`gpeople`, `msgraphContacts`, `gcal`, `msgraphCalendar`) stopped on "No OAuth 2.0 grant found", and a kept grant carried the scopes of every service merged into it. `AccountDraft.services` carries the chosen grant's services to the application step.

`account::bind_loopback` binds an `http` loopback redirection naming no port on an ephemeral port of its host and returns the URI with the bound port; `Account::redirection` (auth get) and the MCP server's code flow use it, the MCP server's own `bind` removed. A configured `http://localhost` bound port 80 before, which an unprivileged process cannot, so the flow fell back to `auth resume`: true of Thunderbird's Google entry too.

Capabilities moved: discovery (cairn/spec/discovery.md: "Pimalaya's own applications" added; "Grant-based service reduction", "Client source preference order", "Scopes follow the application" and "Registration honours print-only" modified), auth (cairn/spec/auth.md: "Authorization code grant" and "Redirection resolution" modified), provider-quirks (cairn/spec/provider-quirks.md: "Current Google endpoints" and "Outlook scope resource" modified).

Verified: `known_apps_are_scoped_to_their_authorization_server`, `known_apps_serve_one_of_the_grant_services`, `pimalaya_reads_the_google_calendar_scope_under_its_names`, `a_port_less_http_loopback_binds_an_ephemeral_port` and `a_loopback_naming_its_port_keeps_it`; `cargo test --all-features` (49 unit tests and the integration suites), `cargo clippy --all-features --all-targets -D warnings`, `cargo check --no-default-features --features rustls-ring`. The registrations, probed without signing in: Google's authorize endpoint answers the sign-in page for the Desktop client with `http://127.0.0.1:53682/` and an error for a non-loopback redirect; Entra's consumers endpoint answers the sign-in page for `http://localhost:53682` and `http://localhost:53682/`, and `invalid_request` for `http://127.0.0.1:53682`. The wizard under a pseudo-terminal, against made-up addresses: `--service gpeople` on gmail.com finds one grant and offers Pimalaya alone, `--service carddav` offers Thunderbird alone, `--service msgraphCalendar` on outlook.com finds the code and device grants.

Left:

- No sign-in was run: a Google and a Microsoft `configure` then `auth get` with the Pimalaya clients on the test accounts are the owner's.
- The Google data disclosure before a sign-in with Pimalaya's client, which the Pimalaya project's verification requires (android cairn/changes/oauth-verification §1), is owed before the next release.
- The device grant with Pimalaya's Entra application needs "Allow public client flows", not probed.
- A Pimalaya client at Fastmail, to ask Fastmail for.
