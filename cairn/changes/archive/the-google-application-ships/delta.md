---
cairn: delta
change: the-google-application-ships
---

# Delta

Capabilities: auth (cairn/spec/auth.md) and mcp (cairn/spec/mcp.md) take one ADDED requirement each; output (cairn/spec/output.md) takes the MODIFIED "A failure carries a stable code"; config (cairn/spec/config.md) the MODIFIED "One sample configuration, the provider accounts in the README"; provider-quirks (cairn/spec/provider-quirks.md) the MODIFIED "Current Google endpoints".

## ADDED Requirements

### Requirement: Pimalaya's Google data disclosure precedes the consent
On an account whose `client-id` is Pimalaya's Google Desktop client, `803561575535-41415ghqi04uibrl6t5umg3p27stdhqr.apps.googleusercontent.com`, and whose grant asks the user's consent (`authorization-code` or `device`), `auth get` SHALL obtain the user's agreement to the disclosure of the Google data the application accesses before anything else: before binding a listener, building the authorization URL or requesting a user code. No other account and no other grant SHALL show it, and `auth listen` and `auth resume`, finishing a flow `auth get` started, show none.

The disclosure SHALL be titled "Pimalaya and your Google data" and list, as aligned fields, one row per Google service the account's scopes reach, all four when the scopes reach none: Gmail (your messages, labels and sending addresses), Google Calendar (your calendars and events), Google Contacts (your contacts and labels), Google Tasks (your task lists and tasks). It SHALL then say that the data is used only to carry out the user's own actions on this device, that Pimalaya runs no server, so the data goes between Google and this device only and is never sold or used for ads, and give the privacy policy, https://pimalaya.org/privacy/.

When stdin and stdout are terminals, `--json` is off and no session reads stdin, `auth get` SHALL print the disclosure on stdout, then `Agree and continue? [y/N] `, and go on only on `y` or `yes`, in any case; any other answer, an empty line or EOF SHALL fail coded `agreement-missing`, nothing sent. `--agree-data-use` SHALL be the agreement given beforehand: no prompt, and the text output SHALL still print the disclosure before the authorization request or the device request; under `--json` it adds nothing to the output. Without it, where no prompt can be asked (`--json`, stdin or stdout not a terminal, a REPL line), `auth get` SHALL fail coded `agreement-missing`, exit 1, before anything is sent, its message holding the disclosure in one sentence, the privacy policy's link and the flag agreeing to it: the disclosure is never skipped silently.

#### Scenario: A script signs in with Pimalaya's Google application
- GIVEN an account on Pimalaya's Google Desktop client and the authorization code grant
- WHEN `ortie --json auth get` runs
- THEN it exits 1 with a failure object coded `agreement-missing`, naming `--agree-data-use` and https://pimalaya.org/privacy/, and no port is bound
- AND `ortie --json auth get --agree-data-use` prints the authorization handoff

### Requirement: The MCP agreement is the server's flag
`ortie mcp` SHALL take `--agree-data-use`, the user's agreement, written in the agent's server configuration, to the disclosure `auth get` shows on Pimalaya's Google application (see the auth capability); no tool argument SHALL give it. Without it, `auth_get` on such an account SHALL answer the failure object coded `agreement-missing`, its message holding the disclosure in one sentence and naming the server flag, before the pending flow is replaced. The `auth_get` answer SHALL carry `disclosure`, that sentence, on such an account, and `null` on any other; the server's instructions SHALL tell the agent to show it to the user with the authorization URI or the user code.

## MODIFIED Requirements

### Requirement: A failure carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, found anywhere in the error chain or read from io-oauth's typed client error or pimalaya-config's account resolution error. A failure with no code SHALL print `"code": null`; the text output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. An OAuth refusal SHALL name its code in its message as the server spelled it (`invalid_grant`), a code io-oauth does not register reading `unregistered code`.

| Code | Raised when |
|---|---|
| `usage-invalid` | the command line, a REPL line or an MCP tool call's arguments do not match the grammar, raised by pimalaya-cli; or Ortie refuses the line after parsing it (a command or a flag the account's grant has no use for, `--handoff` among them, an `auth listen` whose `--redirect-uri` or handoff `redirectUri` is no loopback redirection to bind, an empty device code, a redirected URI that does not parse), exiting 2 as well, and the MCP `auth_resume` refusing such a URI as a tool error; nothing ran |
| `permission-denied` | the grant was refused to the user or the client: `access_denied` at the authorization redirect or in the device flow, `authorization_declined` (Microsoft Entra), a bare HTTP 403 |
| `rate-limited` | a bare HTTP 429; Ortie retries nothing |
| `grant-invalid` | `invalid_grant`: the authorization code, the refresh token or the device code was refused, has expired or was revoked; run `auth get` again |
| `client-invalid` | `invalid_client`: the client secret, certificate or assertion was refused |
| `client-unauthorized` | `unauthorized_client`: the client may not use the grant |
| `scope-invalid` | `invalid_scope` |
| `device-code-expired` | the device code expired before the user approved it, by the server's `expired_token` or by the local deadline |
| `state-mismatch` | the authorization redirect carries no `state`, or another than the request sent |
| `token-missing` | the account stores no token (its read command prints nothing, exiting 0, or non-zero without a word), or no refresh token where a refresh needs one; run `auth get` |
| `token-expired` | the `token` row of `account check`: the stored token expired and holds no refresh token, on a grant other than the client credentials ones, which run again by themselves; run `auth get` |
| `endpoint-missing` | the account configures no endpoint the grant needs (authorization, token, device authorization, revocation); exit 1 |
| `redirection-unsupported` | the `endpoints.redirection` an `auth listen` without `--redirect-uri` would bind is not an http URI on a loopback host; exit 1, `auth resume` with the redirected URI finishing the flow |
| `agreement-missing` | the user did not agree to the Google data disclosure of Pimalaya's application: declined at the prompt, or no `--agree-data-use` where no prompt can be asked (`auth get`, or `ortie mcp` for `auth_get`); exit 1, nothing sent |
| `account-missing` | no account has the name `-a` gave, `default` included |
| `account-default-missing` | no account is marked `default = true`, and `-a` named none |
| `account-default-several` | several accounts are marked `default = true`, and `-a` named none |
| `stdin-taken` | a command asked for its input on stdin (`--handoff -`) inside a REPL or MCP session, which reads its own lines there |

Any other OAuth error code (`invalid_request`, `unsupported_grant_type`, `unsupported_token_type`, `invalid_target`, `server_error`, `temporarily_unavailable`, an unknown one) SHALL carry no code.

#### Scenario: A revoked refresh token
- GIVEN an account whose stored refresh token the server answers `invalid_grant`
- WHEN `ortie --json token refresh` runs
- THEN it exits 1 printing a failure object coded `grant-invalid`, its `error` naming `invalid_grant`

### Requirement: One sample configuration, the provider accounts in the README
The repository SHALL ship config.sample.toml, and the release archives and the Nix package SHALL install it as share/doc/ortie/config.sample.toml, which install.sh copies under its prefix. It holds one `[accounts.example]` table with dotted keys and no nested table, `tls.client` included. Only `default = true`, `client-id` and the storage commands, the keys an account needs to load, SHALL be active. Every other field the configuration reads SHALL be commented as `#key = value` on one line, at its default or an example, the other values of an enumeration listed in brackets at the end of its comment rather than as more lines. Every key SHALL end in one tab-aligned comment, and every command SHALL be a failing `exit 127` placeholder. The client keys SHALL be headed by one comment line pointing to the README for the Google, Microsoft and Fastmail accounts. The file SHALL load as written.

The README SHALL carry one subsection per provider under Configuration (Google, Microsoft, Fastmail), each holding its caveats and one fenced toml block per account, a whole `[accounts.<name>]` table to paste: `google`, `microsoft-graph` and `microsoft-imap-smtp` (one account per resource, since Entra issues a token for one), and `fastmail`. Each block SHALL hold the client, the endpoints, the redirection, the resources and the scopes the provider needs, on Pimalaya's application (at Google its Desktop client id and client secret, with the `mail.google.com`, `calendar`, `contacts` and `tasks` scopes), and storage commands written with `pass` as the example (`pass show ortie/<name>`, `pass insert -m -f ortie/<name>`).

The test suite SHALL parse config.sample.toml through the loader `-c` uses as written, then once per rank with every line uncommented, each field set to its value of that rank (its lines in order, each followed by its bracketed values, a secret's `command` and `raw` being one field), checking each time that `example` is the one account and the default and that every grant takes its turn. It SHALL parse each toml block of the README through the same loader, checking it holds the one account it names and that the README names the four accounts above, so a configuration change that breaks either fails the build.

#### Scenario: A field is renamed
- GIVEN the README's Google block, which sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the block is not updated
- THEN the test suite fails naming the README line of the block and the account `google`

### Requirement: Current Google endpoints
Configs SHALL use `https://accounts.google.com/o/oauth2/v2/auth` and `https://oauth2.googleapis.com/token`; the legacy pair can be rejected at consent with "This app is blocked". Google publishes no `registration_endpoint`: the README's account takes Pimalaya's Desktop client, in a Google project in production pending Google's verification (an unverified-app warning at consent and a 100-user cap until then), registered for `https://mail.google.com/` (Gmail, IMAP and SMTP), the whole `calendar` (the Calendar API and CalDAV), `contacts` (the People API) and `tasks`, not for CardDAV, whose `carddav` scope needs an application of the user's own. Google refresh tokens require `extras.access_type = "offline"`, not an `offline_access` scope. CardDAV and the People API take two scopes that are not interchangeable, `auth/carddav` and `auth/contacts`. An application of the user's own is a Desktop client, whose refresh tokens expire after 7 days while its consent screen stays in testing.

## REMOVED Requirements

None.
