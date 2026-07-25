---
cairn: spec
capability: discovery
status: current
---

# Discovery

The discovery wizard is the natural first contact with Ortie: bare `ortie` runs it (an alias of `auth discover`). It resolves OAuth 2.0 services for an email address or issuer and emits a ready-to-use account config. It leans on io-pim-discovery for resolution and io-oauth for client registration.

### Requirement: Wizard is the default command
Bare `ortie` (no subcommand) SHALL run the discovery wizard; `ortie auth discover` remains the explicit spelling.

### Requirement: Prints, never writes
The wizard SHALL NOT write any file. It prints a complete, valid `[accounts.<name>]` TOML fragment on stdout with its guidance embedded as leading comments, while prompts render on stderr, so `ortie >> <config>` is the write-back. Under `--json` it emits a JSON object instead so scripts can consume the discovery.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable after filling only the marked fields: an `[accounts.<name>]` header (name prompted, input domain or host suggested), an optional client-id placeholder, a commented `default = true`, the discovered grant and endpoints and scopes, and a pass-style storage template. Unresolved issuers appear as a comment line (or JSON issuer key).

### Requirement: Grant-based service reduction
Discovery SHALL reduce services to deduplicated OAuth 2.0 grants tagged with the services sharing them, and present the grant as the unit of choice.

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the provider advertises a `registration_endpoint`, well-known public applications registered against the same authorization server, then a custom entry. The wizard probes each endpoint host for a `registration_endpoint` to decide whether to offer registration.

### Requirement: Registration honours print-only
Dynamic registration SHALL run at wizard time with `token_endpoint_auth_method` none, grant and response types from the discovered grant, the discovered scopes, and client name `Ortie`. The issued client id (and client secret as the config secret shape) land inside the printed fragment. A failed registration reports through its spinner and falls back to the remaining pick-list entries.

#### Scenario: Provider without registration
- GIVEN Google or Microsoft, which publish no `registration_endpoint`
- WHEN the wizard reaches the application step
- THEN the pick list leads with the pre-registered Thunderbird public application, not dynamic registration
