---
cairn: spec
capability: discovery
status: current
---

# Discovery

The configuration wizard is the natural first contact with Ortie: bare `ortie` runs it. It resolves OAuth 2.0 grants for an email address or issuer, walks the user to a complete account, and emits it as a ready-to-append config fragment. It runs no grant of its own: `auth get` authorizes the account afterwards. It leans on io-pim-discovery for resolution and io-oauth for client registration.

### Requirement: Wizard is the default command
Bare `ortie` (no subcommand) SHALL run the wizard, prompting for the input. There is no separate `auth discover` subcommand.

### Requirement: Welcome banner frames the wizard
Bare `ortie` SHALL open with a welcome banner on stderr, before the first prompt, in three short paragraphs: the greeting, what Ortie is (named without reference to the tools that consume its tokens), and what the wizard is about to do, closing on the documented sample configuration as the escape hatch for writing an account by hand. The banner renders on stderr so it never pollutes a redirected fragment, and it is skipped under `--json`. It SHALL NOT suggest a stdout redirection, which can only be set up before the command is run.

### Requirement: Input orients the flow
A single prompt SHALL accept an email address, a bare domain, or an issuer URL, and nothing else. An email or bare domain runs io-pim-discovery's parallel discovery; an issuer URL resolves that issuer's RFC 8414 authorization server metadata into the grants it advertises. The wizard SHALL NOT offer any hand-entry of OAuth 2.0 endpoints.

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline so a single unreachable endpoint cannot stall the interactive wizard. Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered.

### Requirement: Grant-based service reduction
Discovery SHALL reduce services to deduplicated OAuth 2.0 grants tagged with the services sharing them, and present the grant as the unit of choice. Two grants SHALL be reduced into one when they are the same flow against the same authorization server, compared by the host of the endpoint that starts the flow rather than by the endpoint URLs: mechanisms disagree on the exact spelling a provider's endpoints are written with, and the pick list must not ask the user to arbitrate between two spellings of one server. The first spelling seen wins the entry's endpoints, which is the highest-priority mechanism's since compose yields its outputs in mechanism-priority order, and the scopes and services of both are merged into it. The entry SHALL be labelled by its flow and the services it authenticates, never by its endpoints.

### Requirement: A single candidate is not a choice
A step whose reduction leaves exactly one candidate SHALL carry it through without prompting. This holds for the grant pick list and for the application pick list alike: a prompt with nothing to arbitrate is noise, and the entry it would have offered is the one the wizard uses.

### Requirement: Issuers resolve to concrete grants
A discovered `OauthIssuer` entry, and a typed issuer URL, SHALL be resolved through the issuer's RFC 8414 metadata into every grant it advertises: the authorization code grant when an authorization endpoint is published, the device authorization grant when a device authorization endpoint is, and both when both are, since RFC 8414 section 2 and RFC 8628 section 4 let a server advertise them side by side and the choice between them belongs to the pick list. Both need the token endpoint, so a document without one advertises nothing. An issuer whose metadata cannot be resolved into any grant SHALL be dropped from the pick list rather than emitted as a bare issuer comment.

### Requirement: One metadata probe per run
The authorization server metadata SHALL be fetched at most once per run, from the hosts of the chosen grant's endpoints, and shared by the steps that need it: its `scopes_supported` widens the scope options, and its `registration_endpoint` decides whether dynamic registration is offered.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the first label of the email domain, bare domain, or issuer host) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key.

### Requirement: Stop when nothing is discovered
When discovery yields no OAuth 2.0 grant for the given input, the wizard SHALL stop with a message stating it could not automatically discover a configuration for the input, and inviting the user to write the account by hand from the documented sample configuration (linked). It SHALL NOT prompt for any endpoint or emit a partial account.

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the authorization server metadata advertises a `registration_endpoint`, well-known public applications registered against the same authorization server, then a custom entry. The decision reads the metadata probed once for the run rather than probing again. The step SHALL run before the scope step, and hand it the scope options its outcome allows. A pick list holding the custom entry alone SHALL be skipped entirely, that entry asking nothing.

### Requirement: A custom application is not prompted for
The custom entry SHALL prompt for nothing: not the client id, not its secret, not the redirection endpoint. Registering an application of one's own is the rare path, and whoever took it is already editing the configuration, so typing those fields into a wizard only to check them in a file afterwards helps nobody. The account SHALL be emitted with everything the wizard did resolve (grant, endpoints, discovered scopes, storage) and an empty `client-id`.

An account left without an application SHALL be explained on stderr immediately before the printed fragment, so the fragment is read knowing what is missing from it. The explanation SHALL stay short: the fields to fill in (`client-id`, plus `client-secret.raw` and `endpoints.redirection` where the provider requires them) and the documented sample configuration, nothing more. It SHALL NOT be told to run `ortie auth get`, which cannot succeed until the client id is filled in.

### Requirement: Scopes follow the application
The scope step SHALL run after the application step, because what a token may request is a property of the application carrying it. The application step decides where the options come from:

- a well-known public application offers exactly the scopes its registration is granted, with the discovered ones selected, so a scope its client id was never verified for cannot reach the authorization request;
- dynamic registration prompts from the authorization server's advertised scopes, widened by the discovered ones, from inside the registration branch, since the scopes travel in the registration request;
- a custom application prompts for nothing and leaves the discovered scopes in the fragment, since nothing exposes the scopes tied to a registration the user made by hand.

#### Scenario: Thunderbird on Google
- GIVEN a Gmail account and the Thunderbird public application
- WHEN the wizard reaches the scope step
- THEN the options are the three scopes Thunderbird is registered for (`https://mail.google.com/`, `https://www.googleapis.com/auth/carddav`, `https://www.googleapis.com/auth/calendar`)
- AND a People API scope, which that client id is not verified for, is not offered

### Requirement: Registration honours print-only
Dynamic registration SHALL run at wizard time with `token_endpoint_auth_method` none, grant and response types from the discovered grant, the discovered scopes, and client name `Ortie`. The issued client id (and client secret as the config secret shape) land inside the printed fragment. A failed registration reports through its spinner and falls back to the remaining pick-list entries.

#### Scenario: Provider without registration
- GIVEN Google or Microsoft, which publish no `registration_endpoint`
- WHEN the wizard reaches the application step
- THEN the pick list leads with the pre-registered Thunderbird public application, not dynamic registration

### Requirement: Storage entry named, not assumed
The storage step SHALL run in two prompts, mirroring the shared credential picker: a strategy (the credential provider CLIs relevant on the running OS, then custom shell commands), then the entry the token lives under, seeded with the account name. The strategies SHALL be ordered by whether their CLI is found on the `PATH`, the installed ones leading, and the ones that are not SHALL say so rather than be hidden: a missing provider is one package install away, and the configuration written for it is correct either way. The order within each group SHALL be the picker's own, most native first. The entry SHALL be used verbatim in both the read and the write command, so a keyring already holding a token for this account is named as it is rather than under a namespace Ortie picked.

### Requirement: Prints, then offers to save
The wizard SHALL emit a bare, valid `[accounts.<name>]` TOML fragment on stdout in every mode, with no leading comments: the guidance that used to head it lives in the stderr welcome banner, and every prompt and spinner renders on stderr. Under `--json` a JSON object is emitted instead so scripts can consume the discovery. When writing to a terminal the wizard SHALL then offer to save the account it just printed, through a prompt naming a single action and defaulting to saving, seeded with `$XDG_CONFIG_HOME/ortie/config.toml`. The prompt SHALL NOT decide whether the account is printed: it has already been, so declining leaves the user with the fragment to place themselves. A redirected stdout and `--json` SHALL stay non-interactive, with no save prompt, so `ortie >> <config>` keeps working.

### Requirement: Saves by appending, never overwriting
When the user accepts the save, the account SHALL be appended to the chosen file, separated from what precedes it, and a missing file (with its parent directory) SHALL be created. An existing file SHALL NOT be overwritten: the fragment is one `[accounts.<name>]` table, so appending adds an account and leaves the accounts and comments already configured untouched, which is what `ortie >> <config>` does by hand. A file that already holds something SHALL have the append confirmed before it happens, naming the path, since it is a file the user already owns; declining SHALL end the wizard without writing, the account having been printed already. An empty or missing file SHALL NOT be confirmed, having nothing to preserve.

### Requirement: Commands emitted as argv where they can be
A command the wizard builds from a known credential provider SHALL be emitted as an exec-style array, so no shell reinterprets an entry name. A shell string SHALL be used only where the command genuinely needs one: the write half of a provider pair that relies on shell features (`$(cat)` on macOS), and any command the user typed by hand. Emitted TOML values SHALL be escaped for the shape they are written in, so a value carrying a quote or a backslash still parses back.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable as printed whenever the wizard obtained an application: an `[accounts.<name>]` header keyed on the derived account name, the client id and secret obtained at the application step, the resolved grant and endpoints and scopes, `auto-refresh`, and the storage commands. Only the fields the user left for later appear blank: the client id of a custom application, and the storage commands when none were named. There is no issuer placeholder, since an unresolved issuer never reaches the fragment. An empty client id SHALL be emitted as an empty placeholder in both output shapes, so the JSON object carries the same key the TOML fragment does.
