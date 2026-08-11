---
cairn: log
id: wizard-himalaya-parity
date: 2026-08-11
---

# Aligned the wizard with the Himalaya wizard

The wizard now follows the same shape as Himalaya's: one prompt, discovery only, a real test, and guidance on stderr.

## What landed

**A welcome banner on stderr.** Bare `ortie` frames what the tool is and what the wizard is about to do before the first prompt, and links the sample configuration. Skipped under `--json`. The framing comments that used to head the printed fragment moved here, so stdout now carries bare TOML.

**One prompt.** The account name prompt is gone; the name is derived from the input (the first label of the email domain, bare domain, or issuer host) and the user renames the table key by hand.

**Discovery only.** The "Enter OAuth 2.0 details manually" pick-list entry and the URL-goes-straight-to-manual path are both removed. A typed issuer URL now resolves through the issuer's RFC 8414 metadata into a concrete grant, and a discovered `OauthIssuer` resolves the same way instead of being printed as a bare issuer comment, which retires the `issuer` field from the fragment. When nothing resolves, the wizard stops and points at config.sample.toml. A hand-registered client id, secret and storage command are still prompted: those are credentials, not server fields, and they mirror Himalaya's credential prompt.

**Time-bounded discovery.** `compose_all_within` with an 8 second deadline replaces `compose_all`, so a black-hole endpoint costs a few seconds rather than the prompt. This needed io-pim-discovery 0.5 (the crate was pinned to 0.3); no other API moved.

**One metadata probe.** The RFC 8414 document is fetched once per run and feeds both the scope options (`scopes_supported`, new) and the dynamic registration decision, which used to probe on its own.

**A grant test before printing.** After the storage step the wizard runs the discovered grant, mints one access token and writes it through the chosen write command, so a wrong client id, endpoint, scope or storage command stops the wizard. Every line it prints renders on stderr, unlike the `auth` tree it mirrors, because the wizard owns stdout for its fragment; that is why it runs its own compact grant rather than calling `auth get`. It skips, with a note, when there is nothing to test or when the redirection cannot be captured locally (Fastmail's private-use scheme).

## Where it lives

The wizard moved out of `src/auth/discover.rs`, which was never an `auth` subcommand, into `src/wizard.rs` plus `src/wizard/{search,client,storage,check}.rs`, mirroring Himalaya's `src/wizard/`. `AuthDiscoverCommand` is gone; `main` calls `wizard::run`.

## Capabilities moved

- **discovery**: seven requirements added (welcome banner, input orients the flow, time-bounded discovery, issuers resolve to concrete grants, one metadata probe per run, account name derived, stop when nothing is discovered, grant tested before printing); three reworded (prints never writes, complete paste-ready fragment, client source preference order).
- **auth**: the overview no longer names a `discover` subcommand, which has not existed since `remove-auth-discover`.

## Not done

Saving the config to a file (Himalaya added it; Ortie stays print-only and user-owned), a refresh leg after the grant test, and retiring the Fastmail quirks, which remain tracked by `discovery-layering`.
