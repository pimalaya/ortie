---
cairn: delta
change: readme-trimmed
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Samples, one per provider
The repository SHALL ship config.samples/<provider>.toml, each holding one complete `[accounts.<provider>]` table with dotted keys and no nested table, ready to append to a configuration. Its active lines SHALL use Pimalaya's application where one exists, and the scopes it leaves out SHALL stand as commented code. One comment line SHALL head each scope group, naming what it serves, and no other comment SHALL appear. config.sample.toml SHALL be one `[accounts.example]` table listing every field and each of its values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. The test suite SHALL parse each sample and config.sample.toml through the loader `-c` uses, as it stands and once per line of commented code swapped in for the active line of its key, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose commented code sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`: an account takes Pimalaya's application, registered for Gmail, IMAP and SMTP, the Calendar API (`calendar.events`, `calendar.calendarlist.readonly`), the People API and Tasks, not for CalDAV nor CardDAV, or an application of the user's own. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope. CardDAV and the People API take two scopes that are not interchangeable, `auth/carddav` and `auth/contacts`; CalDAV takes the whole `auth/calendar`. An application of the user's own is a Desktop client, whose refresh tokens expire after 7 days while its consent screen stays in testing.

### Requirement: Outlook scope resource
IMAP/POP/SMTP scopes SHALL use the `https://outlook.office.com/` resource (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), never `outlook.office365.com` (a mail host, not a scope resource, rejected with `invalid_scope`). Microsoft publishes no `registration_endpoint`. Pimalaya's application is registered with `http://localhost`, any port, and refuses `127.0.0.1`. Graph needs a Graph-registered client, such as Pimalaya's, and Graph scopes, in an account of its own since Entra issues a token for one resource. The device grant uses `.../oauth2/v2.0/devicecode` with the matching `.../token`. An application of the user's own registers `http://localhost` under the "Mobile and desktop applications" platform, which an organisation refusing Pimalaya's application needs.

## REMOVED Requirements
