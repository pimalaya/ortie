# OAuth 2.1 readiness and multi-grant plan

Living plan for evolving ortie from a single-flow (authorization code grant) OAuth 2.0 CLI into a multi-grant, OAuth 2.1-ready token manager. Iterated on together; landed work moves to the Landed section at the bottom.

Reference: [draft-ietf-oauth-v2-1-15](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-15) (expires September 2026, requirements considered stable).

## Where ortie stands

One grant flow (authorization code, optional PKCE, optional client secret), token show/inspect/refresh over external storage commands, issue/refresh hooks, and an auth discover wizard whose output is print-only. io-oauth already ships everything needed for the device grant (RFC 8628) and dynamic client registration (RFC 7591); io-pim-discovery already resolves RFC 8414 issuer metadata and can enable RFC 9728. The gaps are all in ortie's config shape, CLI surface and wiring.

## Versioning: staying in v1.x

No major release is needed. The library target is removed in a minor: nobody consumes it (io-oauth is the library story; the README and CHANGELOG redirect the rare lib user there), and the flat config design below parses every v1 config unchanged. Two behavioral changes ride the next minor, documented in the CHANGELOG: the pkce default flips to S256 (servers without PKCE support need an explicit `pkce = false`) and auth discover output becomes a config-shaped fragment. Everything else is additive.

What deliberately keeps working: v1 config files parse unchanged (`grant` defaults to authorization-code, endpoints.authorization merely becomes optional, `pkce = true`/`false` still accepted), every v1 CLI invocation keeps its shape (auth get/resume and token show/inspect/refresh, no new required flags), and the token storage JSON and hook environment variables are untouched. The only change an existing account can notice is PKCE-by-default.

Note that the token side (show/inspect/refresh, storage commands, hooks) is already grant-agnostic: every grant ends in the same token response shape. That is why multi-grant support costs nothing downstream of issuance.

## OAuth 2.1: requirement mapping

| 2.1 requirement | ortie today | action |
|---|---|---|
| PKCE (S256) required on every authorization code flow | opt-in, default off | M1 richer pkce config, default flips to S256 |
| Implicit grant removed | never supported | none |
| Resource owner password credentials removed | never supported | none |
| Exact redirect URI matching (loopback port may vary) | compliant; ephemeral 127.0.0.1 port is the permitted loopback exception | M3 doc note only |
| Refresh token rotation or sender-constraining for public clients | rotation handled; old refresh token kept when server omits a new one | M3 verify failure ordering |
| No bearer tokens in query strings | never done | none |
| state optional once PKCE is on | always generated | keep; harmless CSRF belt-and-braces |

Conclusion: ortie has no 2.1-forbidden behavior to remove. Readiness is (a) making PKCE the default posture, (b) supporting the grants 2.1 keeps (authorization code, device via RFC 8628, refresh), and (c) leaning on RFC 8414 metadata so servers can tell us what they support.

## Design decisions

### D1: grant is config; flat selector + per-command endpoint checks

The grant is a flat account field; auth get runs whatever grant the account declares. No CLI option.

```toml
[accounts.example]
grant = "authorization-code" # the default when omitted; or "device"
endpoints.authorization = "https://accounts.example.com/authorize"     # authorization-code only
endpoints.device-authorization = "https://accounts.example.com/device" # device only
endpoints.token = "https://accounts.example.com/token"                 # both grants, plus token refresh
#endpoints.redirection = "http://localhost"                            # authorization-code only, optional
```

All endpoints are optional at parse time; each command checks the ones it actually needs and fails with an error naming the missing field. This capability-style late binding fits a CLI whose commands have very different needs: token show works with client-id + storage alone, token refresh needs only the token endpoint, only auth get needs the grant's endpoints. Bonus: v1 configs parse unchanged, since grant defaults to authorization-code and every endpoint name stays put.

Rejected: a strictly-typed tagged grant table (`[accounts.<name>.grant]` with `type = "..."`); it reads heavier, its promised parse-time strictness is weaker than it looks with serde+TOML (internally tagged enums buffer their content, losing line/span info in errors, and deny_unknown_fields stops working on them), and it rejects minimal storage-only accounts that token show serves happily. Also rejected: a `--grant` CLI flag (the config declares intent, commands just run it), inferring the grant from which endpoints are present (implicit magic; the selector's default covers the common case).

Trade-off accepted: a provider offering both grants means two accounts, which can share the same storage commands.

### D2: pkce config shape

Recommended: widen the existing boolean in place. The field applies to the authorization code grant only; the device grant has no PKCE and ignores it.

```toml
pkce = true    # S256 (the default when omitted)
pkce = "s256"  # explicit
pkce = "plain" # escape hatch for broken servers
pkce = false   # opt out entirely
```

`true` means S256, deserialized via an untagged bool-or-string shape so existing configs keep parsing. The default when omitted becomes S256: the headline 2.1 alignment; servers that reject PKCE params need an explicit `pkce = false`.

### D3: modeling 2.1 vs 2.0

Recommended: no `version` config knob. 2.1 is a constraint profile of 2.0, not a new wire protocol; the token and authorization messages are identical. Ortie adopts 2.1-compatible defaults with explicit 2.0 escape hatches (`pkce = false`, `pkce = "plain"`). io-oauth supports OAuth as a set of RFCs, organised one module per RFC (rfc6749, rfc7636, rfc7591, rfc8628) with no version wrapper, so there is no version module for ortie to track; the RFC set it implements already covers what the 2.1 draft consolidates. If a hard-strict mode ever proves useful (refuse plain PKCE, refuse no-PKCE, refuse token endpoints over plain HTTP), it can land later as a single `strict = true` flag.

### D4: extras passthrough

Recommended: a raw account-level table forwarded verbatim into the configured grant's initiation request: the authorization URL query for the authorization code grant (io-oauth Oauth20AuthorizationRequestParams.extras), the device authorization request body for the device grant.

```toml
[accounts.example.extras]
access_type = "offline"                   # Google: required for a refresh token
prompt = "consent"                        # Google
resource = "https://api.fastmail.com/"    # RFC 8707; Fastmail hard-requires it
login_hint = "user@example.com"
```

Keys are wire parameter names and must NOT be kebab-renamed by serde; values are strings. This one table unblocks Google refresh tokens, Fastmail (RFC 8707) and Entra tenants without ortie learning any provider-specific logic.

### D5: device grant CLI shape

Recommended: auth get and auth resume keep their names and dispatch on the account's configured grant (D1). Authorization code path: unchanged. Device path: request device authorization, print user_code and verification_uri (prefer verification_uri_complete when present), then poll via io-oauth await_device_access_token, write storage, fire on-issue hooks. Non-interactive or `--json`: print the device authorization response and exit; polling is then resumed with auth resume, whose positional input is interpreted per the account's grant: the redirected URI on an authorization-code account, the device code on a device account. The `--state`/`--pkce`/`--redirect-uri` flags are authorization-code-only and rejected on device accounts.

### D6: wizard prints, never writes (settled 2026-07-13)

The wizard is the default command (bare `ortie`, alias of auth discover) and never writes any file: it prints a complete `[accounts.<name>]` fragment as valid TOML on stdout, guidance embedded as leading comments, while prompts render on stderr (inquire's default). The user's shell is the write-back: `ortie >> <config>` appends the fragment directly, and `--json` swaps it for a JSON object so scripts and other tools can consume the discovery. Completeness is what makes paste-yourself painless: the wizard asks the minimum questions (input, service pick, account name with the input's domain suggested, optional client id) and emits a pass-style storage template plus commented default / client-id spots, so the paste is runnable after filling the marked fields. The grant is chosen during discovery (M4: metadata-driven grant choice).

Rejected: toml_edit write-back into the config file. It would create a maintenance contract (name collisions, comment preservation, which of several -c paths to write, and eventually an account list / configure command tree) for no gain over the shell append. Config files stay entirely user-owned, matching ortie's token-storage philosophy; the previously open which-path-to-write question disappears with it. Himalaya keeps its own field-by-field writing wizard: different audience (porcelain onboarding), justified divergence.

## Milestones

All milestones ship within v1. M0 through M3 make up the next minor release (M0 and M1 landed, and M5 jumped the queue into it; see the Landed section); M4 and M6 follow as further minors, each independently releasable.

### M2: device authorization grant end-to-end

- Add the endpoints.device-authorization config field (deferred from M1) and its Account counterpart.
- auth get dispatches on the configured grant (D5); `grant = "device"` becomes runnable: device authorization request, user-code display, polling loop, storage write, on-issue hooks (shared with the code grant path). The grant = "device" bail placeholders in auth get / auth resume go away.
- auth resume interprets its positional per the account's grant: redirected URI (authorization code) or device code (device); authorization-code-only flags rejected on device accounts.
- No io-oauth work needed; everything exists.

### M3: release polish

- Verify refresh rotation ordering: a rotated refresh token must be persisted before the old one can be lost (audit the write-after-refresh window in token refresh).
- README section on 2.1 posture: grants supported, PKCE default, exact redirect matching with the loopback variable-port exception, rotation behavior.
- CHANGELOG notes: pkce default flip (opt out with `pkce = false`), discover output shape, lib removal; no config migration needed.
- Re-check the final RFC (if published by then) for anything drafts did not carry. Release the minor.

### M4: discovery upgrades

- Enable io-pim-discovery's rfc9728 feature.
- `auth discover <URI>`: resolve RFC 8414 issuer metadata first (ComposeClientStd::oauth_server), fall back to RFC 9728 resource metadata then its authorization servers, fall back to manual entry pre-seeded with the issuer. No more straight-to-manual on `://` inputs.
- Surface grant choice from metadata: grant_types_supported and device_authorization_endpoint drive which grants are offered; code_challenge_methods_supported drives the suggested pkce value.
- Unresolvable bare OauthIssuer picks get a metadata-resolution retry instead of dead-ending.

### M6: token revocation, RFC 7009

- io-oauth first: revocation request coroutine + Oauth20ClientStd method.
- ortie: optional endpoints.revocation (prefilled by the wizard from metadata revocation_endpoint) and a `token revoke` command (revokes the refresh token when present, else the access token, per RFC 7009 recommendation), clearing or overwriting storage afterwards.

## Backlog (not scheduled)

- Wizard: emit the extras a provider is known to require into the fragment (Fastmail's RFC 8707 resource, Google's access_type=offline), so a dynamically registered Fastmail account works without a manual D4 edit.
- auth get: skip the loopback listener when the configured redirection is not an http URL (private-use scheme registrations) and point at auth resume instead of erroring after the browser opened.
- Optional strict mode (refuse plain PKCE, refuse token endpoints over plain HTTP).
- Client credentials grant: kept by 2.1, missing in io-oauth; machine-to-machine is off ortie's user-token focus for now.
- RFC 7662 introspection: a `token inspect --remote` complement to the local metadata view.
- `--dns` / `--tls` flags on auth discover, mirroring the io-pim-discovery CLI.

## Landed

### M0: pure CLI binary (2026-07-13)

- Removed src/lib.rs and the cli / client cargo features; the binary always builds and its entry point declares mod cli. Remaining features: TLS providers (rustls-ring default), vendored, notify. Feature-gated deps became plain deps; build.rs lost its cfg split; the bin target lost required-features.
- Cargo.toml description is now "CLI to manage OAuth 2.0 tokens"; dropped the docs.rs metadata, the documentation field, the api-bindings category and the io-free/coroutine keywords.
- README: dropped the docs.rs badge; the Usage > Library section states ortie exposes no library API and redirects to io-oauth.
- Cleanup exposed by the lib removal: deleted five never-called pub helpers (auth_get url_parser / state_parser / pkce_code_verifier_parser, auth_resume serialize_state / serialize_pkce_code_verifier) and the never-read Account.default field; converted the no_std-style alloc:: / core:: imports to std.

### M1: config schema + extras wiring (2026-07-13)

- Added the `grant` selector (GrantConfig, default authorization-code); grant = "device" parses but auth get / auth resume bail until M2 lands.
- endpoints.authorization / token / redirection are all optional at parse time; auth get checks authorization, auth resume and token refresh check token, token show needs none.
- Widened pkce to bool-or-string via a hand-rolled serde visitor (true / "s256" = S256, "plain", false = off); the omitted default is now S256.
- Added the extras account table (keys verbatim, no kebab renaming), wired into the authorization request query via io-oauth Oauth20AuthRequestParams.extras.
- Reshaped auth discover output into a paste-ready account config fragment (grant, endpoints.*, scopes; unresolved issuers as a comment line / JSON issuer key).
- Updated config.sample.toml, README and CHANGELOG.
- Deviation from the original M1 scope: endpoints.device-authorization moved to M2; landing it without its reader would leave a parsed-but-never-read field (dead-code warning), and M1 + M2 ship in the same release anyway.
- Context: io-oauth stabilized at 0.1.0 with a per-RFC layout (rfc6749 / rfc7591 / rfc7636 / rfc8628, no v2_0 wrapper) mid-milestone; ortie now builds against the published crate, no local patch.

### Wizard as default command + complete fragment (2026-07-13)

- Bare `ortie` runs the discovery wizard (the root parser's subcommand became optional); `ortie auth discover` remains the explicit spelling.
- The fragment is complete and appendable per D6: pure valid TOML on stdout with guidance as leading comments, an `[accounts.<name>]` header (name prompted, input's domain or URI host suggested, quoted when not a bare TOML key), an optional client id prompt (commented placeholder pointing at the README when left empty), a commented default = true, and a pass-style storage template. Prompts render on stderr (inquire), and the no-service narration moved off stdout, so `ortie >> <config>` appends cleanly.
- README (Features, Configuration, Usage > Discover an account), config.sample.toml header and the CHANGELOG wizard entry updated; the previously open which-path-to-write question is void.

### Source tree and docs refresh (2026-07-13)

- Inlined the cli/ folder: src/cli.rs (root parser), src/config.rs, src/account.rs, and the command trees src/auth.rs + src/auth/{discover,get,resume}.rs and src/token.rs + src/token/{show,inspect,refresh}.rs.
- src/main.rs now carries the architecture document in its header, the way lib.rs does for io-oauth; written in paragraphs, no dash lists.
- Every pub item carries a doc comment, first paragraph two lines at most: clap renders it as the `-h` summary and the remaining paragraphs as the `--help` page (verified on the built binary).
- CONTRIBUTING rewritten for the pure-CLI reality (paragraph-style layout description, published-crates default with a patch-locally recipe); README intro converted to a paragraph and its tagline aligned with the Cargo description (CLI to manage OAuth tokens); docs/README.md index added.
- CHANGELOG [Unreleased] compacted from a history log into a net diff against 1.1.0 (interior churn like the wizard output reshape and the intermediate cli-layer reorganisation folded into their final-state entries).

### M5: dynamic client registration in the wizard (2026-07-15)

- Landed ahead of M2 and M4, nothing in it depended on them. The application step now offers every way to obtain a client, sorted by a new io-oauth preference order: dynamic registration when the provider advertises it, well-known public applications registered against the same authorization server, then the custom entry. The list shows even when no known application matches, so the straight-to-custom fall-through became [Dynamic registration, Custom application].
- Detection: no discovery mechanism hands the wizard registration support (the compose layer drops registration_endpoint when resolving issuers, and the fixed provider rules and autoconfig/ISPDB sources never see RFC 8414 metadata), so the wizard probes at the application step: issuer guessed from each endpoint host (https://<host>), metadata fetched via ComposeClientStd::oauth_server, entry hidden without a registration_endpoint. Google and Microsoft publish none, so their lists keep leading with Thunderbird; Fastmail advertises one. Rejected: extending the compose AuthMethod variants to carry registration_endpoint; it covers metadata-sourced entries only and ripples through every wizard consumer (himalaya, cardamum, calendula).
- io-oauth grew rfc7591::source::Oauth20ClientSource (dynamic registration, public client, manual; declaration order is the preference order, the pick list sorts by it) and Oauth20ClientStd::register_client, inlining the RFC 7591 coroutine like the other per-operation methods; the std client moved to the crate-root client module along the way, since it spans the RFC modules. It keeps its version-scoped name and version-less methods (version-prefixed methods rejected as heavy); a future OAuth version would add a sibling client, unified behind a version-agnostic OauthClientStd wrapper only once one exists. Ortie path-patches io-oauth until the next release.
- Registration runs at wizard time, keeping the print-only philosophy: token_endpoint_auth_method none, grant_types and response_types from the discovered grant, the discovered scopes, client_name Ortie; the issued client_id (and client_secret as the config secret shape) land inside the fragment. Deviation from the original design, found in cardamum's live notes: Fastmail rejects every http redirection at registration, loopback included, so the wizard registers http://127.0.0.1 first and retries with org.pimalaya.ortie://redirect on invalid_redirect_uri, pinning endpoints.redirection so auth resume finishes the flow by hand. A failed registration reports through its spinner and falls back to the remaining pick-list entries.
- The grant step was relabeled while touching the wizard: discovery always reduced services to deduplicated grants tagged with the services sharing them, and the prompts now say so ("Choose an OAuth 2.0 grant:", "Found N OAuth 2.0 grant(s)").
