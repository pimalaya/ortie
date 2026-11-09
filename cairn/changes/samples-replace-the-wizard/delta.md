---
cairn: delta
change: samples-replace-the-wizard
---

# Delta

Capabilities: config (cairn/spec/config.md) takes both ADDED requirements, "Account resolution failures name what is missing" and the first seven REMOVED; output (cairn/spec/output.md) takes "Commands carrying a schema", "Shared shapes and spellings", "JSON keys are camelCase" and "Only Ortie's own types are described"; repl (cairn/spec/repl.md) takes "Command grammar reuse"; provider-quirks (cairn/spec/provider-quirks.md) takes "Current Google endpoints", "Outlook scope resource" and "Fastmail resource and redirect"; discovery (cairn/spec/discovery.md) loses every requirement, and its file goes.

## ADDED Requirements

### Requirement: A bare invocation shows the help
A bare `ortie` SHALL show the help, whether a configuration exists or not, and so SHALL `--account` with no subcommand, a half-typed command. Nothing prompts: Ortie writes no configuration.

### Requirement: Samples, one per use case
The repository SHALL ship samples/, one file per use case, each holding one complete `[accounts.<name>]` table ready to append to a configuration, and the test suite SHALL parse each through the loader `-c` uses, so a configuration change that breaks a sample fails the build. config.sample.toml stays the reference of every field.

#### Scenario: A field is renamed
- GIVEN a sample setting `endpoints.redirection`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

## MODIFIED Requirements

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for and points at the samples, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: Commands carrying a schema
Every command printing a value SHALL carry one key: `ortie-token-show`, `ortie-token-inspect`, `ortie-token-refresh`, `ortie-token-revoke`, `ortie-auth-get`, `ortie-auth-listen` and `ortie-auth-resume`, beside the `completion`, `manual` and `json-schema` keys pimalaya-cli registers. Bare `ortie` carries none, showing the help, and neither does the REPL: it dispatches the `token` and `auth` leaves and prints their payloads unchanged, so those keys describe its lines too. `ortie-error` describes the failure object any command prints instead of its payload.

### Requirement: Shared shapes and spellings
`token inspect` SHALL print `issuedAt` as an RFC 3339 instant in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

### Requirement: Command grammar reuse
Each stdin line SHALL be split as a shell splits it, quotes and escapes included, then parsed with the same `token` and `auth` command grammar as the one-shot CLI. A line whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`. Configuration types SHALL stay kebab-case: what the loader reads is not what the printer emits.

### Requirement: Only Ortie's own types are described
Every registered command payload SHALL be owned by Ortie. No schema SHALL be derived from an io-oauth type, so the machine contract stays a decision of this repository and needs no release of the library below it. The failure object is the family's shared `ErrorReport` from pimalaya-cli.

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`: an account takes Pimalaya's application, registered for Gmail, IMAP and SMTP, the Calendar API (`calendar.events`, `calendar.calendarlist.readonly`), the People API and Tasks, not for CalDAV nor CardDAV, or an application of the user's own. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope. CardDAV and the People API take two scopes that are not interchangeable, `auth/carddav` and `auth/contacts`; CalDAV takes the whole `auth/calendar`.

### Requirement: Outlook scope resource
IMAP/POP/SMTP scopes SHALL use the `https://outlook.office.com/` resource (`IMAP.AccessAsUser.All`, `POP.AccessAsUser.All`, `SMTP.Send`), never `outlook.office365.com` (a mail host, not a scope resource, rejected with `invalid_scope`). Microsoft publishes no `registration_endpoint`. Pimalaya's application is registered with `http://localhost`, any port, and refuses `127.0.0.1`. Graph needs a Graph-registered client, such as Pimalaya's, and Graph scopes, in an account of its own since Entra issues a token for one resource. The device grant uses `.../oauth2/v2.0/devicecode` with the matching `.../token`.

### Requirement: Fastmail resource and redirect
The Fastmail authorize endpoint SHALL receive the JMAP session URL as RFC 8707 resource and a non-empty scope, or it bounces pre-consent; the session's RFC 9728 metadata names the resource. It forwards every authorization parameter to a strict check, so none beyond RFC 6749, PKCE, `resource` and `login_hint` is sent. Its dynamic registration answers the same client id to the same request and accepts only a reverse-DNS private-use redirect scheme (`org.pimalaya.ortie://redirect` in the sample), refusing every http/loopback redirect. A desktop browser cannot hand the private-use scheme back to Ortie, so `auth get` prints the manual `auth resume`; on mobile the OS routes the scheme back to the app.

## REMOVED Requirements

### Requirement: A named command runs the wizard
### Requirement: The offer is a hook, not a gate
### Requirement: A bare invocation meets the newcomer, not the user
### Requirement: The welcome names the missing path
### Requirement: A generated account takes a free name and one default
### Requirement: The wizard saves where the configuration lives
### Requirement: A defaulted field is absent from the generated account
### Requirement: One entry into the wizard
### Requirement: Welcome banner frames the wizard
### Requirement: Input orients the flow
### Requirement: Discovery is time-bounded
### Requirement: Grant-based service reduction
### Requirement: Resource-bound grants
### Requirement: A single candidate is not a choice
### Requirement: Issuers resolve to concrete grants
### Requirement: One metadata probe per run
### Requirement: Metadata-driven PKCE
### Requirement: Revocation endpoint prefilled
### Requirement: Account name derived, not prompted
### Requirement: Stop when nothing is discovered
### Requirement: Client source preference order
### Requirement: A custom application is not prompted for
### Requirement: Scopes follow the application
### Requirement: Pimalaya's own applications
### Requirement: Registration honours print-only
### Requirement: Storage entry named, not assumed
### Requirement: Saves, appends, or prints
### Requirement: Saves by appending, never overwriting
### Requirement: Commands emitted as argv where they can be
### Requirement: Complete, paste-ready fragment
