---
cairn: change
id: wizard-himalaya-parity
status: landed
created: 2026-08-11
---

# Align the wizard with the Himalaya wizard

## Why

Himalaya's wizard and Ortie's wizard solve the same problem for two different domains: take one email address, find what the provider offers, collect the one secret the user owns, prove it works, print a config. Himalaya has since sharpened that flow into a shape Ortie has not followed:

- it opens with a welcome banner on stderr, so bare `himalaya` explains what the tool is and what the wizard is about to do before the first prompt;
- it asks for the input and nothing else, deriving the account name instead of prompting for it;
- it is discovery-only: no hand-entry of server fields, and when discovery finds nothing it stops and points at the documented sample rather than emitting a half-config;
- it bounds discovery with a deadline, so one black-hole endpoint cannot stall an interactive prompt;
- it tests the account before printing, so a config that cannot work never reaches the user;
- its guidance lives on stderr, leaving stdout carrying nothing but the config.

Ortie prompts twice (input, then account name), lets the user hand-type OAuth endpoints, runs discovery unbounded, never proves the result works, and heads its stdout fragment with a block of comments duplicating what a banner should say. The three that matter most are the missing test (the wizard's whole promise is a config that works), the manual-entry path (it produces accounts that were never discovered and cannot be validated), and the double prompt.

The domains differ in exactly one place: Himalaya tests by opening a connection with the credential it just collected, which is silent and instant. Ortie's equivalent is running the discovered grant, which opens a browser or prints a user code. That is interactive by nature, but it is still the only thing that proves the client id, endpoints, scopes, PKCE posture and write-storage command all agree.

## What

Rework the wizard into `src/wizard.rs` and `src/wizard/`, mirroring Himalaya's `src/wizard/`, and change the flow to:

1. **Welcome banner on stderr.** Frames Ortie, states what the wizard does, links the sample config. Skipped under `--json`.
2. **One prompt.** An email address, a bare domain, or an issuer URL. The account name is derived from it (the domain's first label) and is no longer prompted; the user renames the table key by hand.
3. **Time-bounded discovery.** `compose_all_within` with an 8 second deadline replaces `compose_all`, which needs io-pim-discovery 0.5 (Ortie is pinned to 0.3, Himalaya already moved).
4. **Discovery only.** The "Enter OAuth 2.0 details manually" pick-list entry and the URL-goes-straight-to-manual path both go away. A typed issuer URL now resolves through the issuer's RFC 8414 metadata into a concrete grant, and a discovered `OauthIssuer` entry resolves the same way instead of printing a bare issuer comment. When nothing resolves, the wizard stops with a message pointing at the documented sample.
5. **One metadata probe.** The RFC 8414 document is fetched once per run and feeds both the scope options (`scopes_supported`) and the dynamic-registration decision, instead of being fetched only for registration.
6. **Grant test before printing.** After the storage step the wizard runs the discovered grant, mints an access token and writes it through the chosen write command. Every line it prints goes to stderr. A failure stops the wizard, exactly as a failed connection stops Himalaya's. It is skipped, with a note, when there is nothing to test (no client id, no storage) or when the redirection cannot be captured locally (a private-use scheme, as Fastmail's dynamic registration mandates), because those cannot complete unattended.
7. **Bare TOML on stdout.** The comment header moves into the stderr banner, so `ortie >> config.toml` appends the fragment alone.

The application step keeps its shape: dynamic registration, well-known public applications, custom entry. A hand-typed client id and secret are the user's own credential, not a server field, so they stay, mirroring Himalaya's credential prompt.

## Non-goals

- Offering to save the config to a file. Himalaya added that; Ortie's contract is print-only and the user keeps owning the config.
- A second refresh leg after the grant test. The grant alone proves the account; the refresh path stays `ortie token refresh`.
- Retiring the Fastmail quirks (`resource` extra, advertised scopes). They remain a stopgap tracked by `discovery-layering`.
