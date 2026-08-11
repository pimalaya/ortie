---
cairn: delta
id: wizard-himalaya-parity
---

# Delta

## ADDED Requirements

### Requirement: Welcome banner frames the wizard
Bare `ortie` SHALL open with a welcome banner on stderr, before the first prompt, stating what Ortie is, what the wizard does, and where every configuration field is documented. The banner renders on stderr so it never pollutes a redirected fragment, and it is skipped under `--json`.

### Requirement: Input orients the flow
A single prompt SHALL accept an email address, a bare domain, or an issuer URL, and nothing else. An email or bare domain runs io-pim-discovery's parallel discovery; an issuer URL resolves that issuer's RFC 8414 authorization server metadata into a grant. The wizard SHALL NOT offer any hand-entry of OAuth 2.0 endpoints.

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline so a single unreachable endpoint cannot stall the interactive wizard. Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered.

### Requirement: Issuers resolve to concrete grants
A discovered `OauthIssuer` entry, and a typed issuer URL, SHALL be resolved through the issuer's RFC 8414 metadata into a concrete grant: authorization code when an authorization endpoint is advertised, device authorization when only a device authorization endpoint is. An issuer whose metadata cannot be resolved into endpoints SHALL be dropped from the pick list rather than emitted as a bare issuer comment.

### Requirement: One metadata probe per run
The authorization server metadata SHALL be fetched at most once per run, from the hosts of the chosen grant's endpoints, and shared by the steps that need it: its `scopes_supported` widens the scope options, and its `registration_endpoint` decides whether dynamic registration is offered.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the first label of the email domain, bare domain, or issuer host) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key.

### Requirement: Stop when nothing is discovered
When discovery yields no OAuth 2.0 grant for the given input, the wizard SHALL stop with a message stating it could not automatically discover a configuration for the input, and inviting the user to write the account by hand from the documented sample configuration (linked). It SHALL NOT prompt for any endpoint or emit a partial account.

### Requirement: Grant tested before printing
The discovered grant SHALL be run before the fragment is printed, so a wrong client id, endpoint, scope or write-storage command stops the wizard instead of yielding a config that cannot mint a token. The test issues one access token and writes it through the chosen write command, and every line it prints renders on stderr. It SHALL be skipped, with a note on stderr, when there is nothing to test (no client id, or no storage commands) or when the account's redirection cannot be captured by the local listener, since those cannot complete inside the wizard.

## MODIFIED Requirements

### Requirement: Prints, never writes
The wizard SHALL NOT write any file. It prints a bare, valid `[accounts.<name>]` TOML fragment on stdout, with no leading comments: the guidance that used to head the fragment now lives in the stderr welcome banner, and every prompt, spinner and test line renders on stderr, so `ortie >> <config>` is the write-back. Under `--json` it emits a JSON object instead so scripts can consume the discovery.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable as printed: an `[accounts.<name>]` header keyed on the derived account name, the client id and secret obtained at the application step, the resolved grant and endpoints and scopes, `auto-refresh`, and the storage commands. Only the fields the user deliberately left empty at the application or storage step appear blank; there is no issuer placeholder, since an unresolved issuer never reaches the fragment.

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the authorization server metadata advertises a `registration_endpoint`, well-known public applications registered against the same authorization server, then a custom entry. The decision reads the metadata probed once for the run rather than probing again.

## REMOVED Requirements

None. "Grant-based service reduction" and "Registration honours print-only" carry over unchanged.
