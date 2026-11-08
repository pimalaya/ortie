---
cairn: delta
change: the-wizard-takes-its-input
---

# Delta

## MODIFIED Requirements

### Requirement: Input orients the flow
A single prompt SHALL accept an email address, a bare domain, or an issuer URL, and nothing else. `configure` MAY be given that input as its positional argument instead, which skips the prompt: the input is then trimmed, validated and discovered exactly as a typed answer is. An email or bare domain runs io-pim-discovery's parallel discovery; a URL resolves its RFC 8414 authorization server metadata into the grants it advertises, or, when it has none, its RFC 9728 protected resource metadata into the grants of the authorization servers it names, each bound to the resource and narrowed to its scopes. The wizard SHALL NOT offer any hand-entry of OAuth 2.0 endpoints.

### Requirement: Grant-based service reduction
Discovery SHALL reduce services to deduplicated OAuth 2.0 grants tagged with the services sharing them, and present the grant as the unit of choice. Two grants SHALL be reduced into one when they are the same flow against the same authorization server, compared by the host of the endpoint that starts the flow rather than by the endpoint URLs: mechanisms disagree on the exact spelling a provider's endpoints are written with, and the pick list must not ask the user to arbitrate between two spellings of one server. The first spelling seen wins the entry's endpoints, which is the highest-priority mechanism's since compose yields its outputs in mechanism-priority order, and the scopes and services of both are merged into it. The entry SHALL be labelled by its flow and the services it authenticates, never by its endpoints.

`--service`, repeatable, SHALL keep only the grants tagged with at least one of the named services, written as the pick list labels them: `imap`, `pop3`, `smtp`, `jmap`, `caldav`, `carddav`, `webdav`, `managesieve`, `gmail`, `gcal`, `gpeople`, `msgraph`, `msgraphCalendar` or `msgraphContacts`. An unknown name SHALL be a usage error listing the valid ones. A grant resolved from a typed URL carries no service, the URL having named what it serves, and SHALL be kept. Without `--service`, every grant is kept.

#### Scenario: A contacts tool hands its setup over
- GIVEN an address whose discovery yields a grant for `imap` and `smtp` and another, on a different authorization server, for `carddav`
- WHEN `ortie configure <address> --service carddav` runs
- THEN the `carddav` grant is carried through without a pick list

### Requirement: A single candidate is not a choice
A step whose reduction leaves exactly one candidate SHALL carry it through without prompting, the `--service` filter being part of the grant reduction. This holds for the grant pick list and for the application pick list alike: a prompt with nothing to arbitrate is noise, and the entry it would have offered is the one the wizard uses.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. `--name` gives it, used exactly as written, never derived nor suffixed. Without it, the wizard derives one from the input (the first label of the email domain, bare domain, or issuer host) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key.

### Requirement: Stop when nothing is discovered
When discovery yields no OAuth 2.0 grant for the given input, or the `--service` filter leaves none, the wizard SHALL stop with a message stating it could not automatically discover a configuration for the input, naming the services when it was filtered, and inviting the user to write the account by hand from the documented sample configuration (linked). It SHALL NOT prompt for any endpoint or emit a partial account.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable as printed whenever the wizard obtained an application: an `[accounts.<name>]` header keyed on the account name, the client id and secret obtained at the application step, the resolved grant and endpoints and scopes, `auto-refresh`, and the storage commands. Only the fields the user left for later appear blank: the client id of a custom application, and the storage commands when none were named. There is no issuer placeholder, since an unresolved issuer never reaches the fragment. An empty client id SHALL be emitted as an empty placeholder in both output shapes, so the JSON object carries the same key the TOML fragment does.

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It SHALL take the wizard's input as an optional positional, the account name as `--name`, and the services to keep grants for as `--service`, repeatable, so another tool can hand its setup over with `ortie configure <ADDRESS> --name <NAME> --service <SERVICE>` and then `ortie auth get -a <NAME>`. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead. The offer a missing configuration raises runs it with none of them.

### Requirement: A generated account takes a free name and one default
A derived account name SHALL be suffixed until the configuration does not already hold it, since a second `[accounts.<name>]` table makes the whole document fail to parse. A name given with `--name` SHALL be taken as written: when the configuration already holds that exact name, the wizard SHALL ask, before any other prompt and defaulting to yes, whether to keep that account. Kept, it writes nothing, prints `{name, default, document}` with `document` `null` and `default` telling whether that account claims the default, and exits 0; declined, it fails naming the account. The generated account claims `default` only when no other account does, since two defaults resolve to whichever the account map yields first.

#### Scenario: A tool hands over an account it already set up
- GIVEN a configuration holding `[accounts.work]`
- WHEN `ortie configure alice@example.org --name work --json` runs and the keep prompt is accepted
- THEN the configuration is unchanged, no discovery runs, and `{"name": "work", "default": true, "document": null}` is printed for a `work` account claiming the default

### Requirement: The wizard saves where the configuration lives
The save SHALL NOT prompt for a path: it writes where `-c` or `ORTIE_CONFIG` pointed, or the default location, and that file is the configuration whose names a derived or given account name is checked against. A file already holding accounts is appended to rather than overwritten, and confirmed before it happens in the one prompt the save step asks, since it is a file the user already owns. Declining SHALL print the account on stdout instead of writing it.

### Requirement: Shared shapes and spellings
`configure` SHALL print `{name, default, document}`, the account name, whether it claims the default, and the TOML fragment its text form prints, every key present; `document` is `null` when `--name` kept an account the configuration already held, the text form then naming the kept account. `token inspect` SHALL print `issuedAt` as an RFC 3339 instant in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.
