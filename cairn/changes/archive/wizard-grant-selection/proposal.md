---
cairn: change
id: wizard-grant-selection
status: landed
created: 2026-08-11
---

# Pick one grant per authorization server, and scopes the application can carry

## Why

Setting up a Gmail account exposes three flaws in the wizard, each small, together enough to hand the user a config that cannot mint a token.

**The same authorization server is offered twice.** Discovery runs every mechanism and reduces their outputs together. Two mechanisms describe Google's authorization code grant: the fixed provider rules carry `https://accounts.google.com/o/oauth2/v2/auth` with `https://oauth2.googleapis.com/token`, while Mozilla's autoconfig entry for gmail still carries the legacy `https://accounts.google.com/o/oauth2/auth` with `https://www.googleapis.com/oauth2/v3/token`, the pair Thunderbird itself hardcodes. Both describe the same server, so the collector merges them into one service config by appending the second's authentication method to the first's list, and the wizard's own reduction keeps them apart because it compares endpoint URLs. The user is asked to arbitrate between two spellings of one thing, with the token endpoint as the only visible difference, and the legacy spelling is the one Google can reject at consent with "This app is blocked".

**Only one grant per authorization server can ever be discovered.** The metadata resolution returns the authorization code grant whenever an authorization endpoint is advertised, and falls back to the device grant only when there is none. RFC 8414 section 2 and RFC 8628 section 4 let a server advertise both side by side, and nothing says a client must choose one on the user's behalf. A headless machine wants the device flow even where a browser redirect is possible, and today it cannot be discovered as long as the server also speaks the redirect flow.

**Scopes are asked before the application that has to carry them.** The scope step runs first, seeded by discovery and widened by the authorization server's `scopes_supported`; the application step then only fills scopes when the scope step left none. So a scope the chosen application was never registered for reaches the authorization request, and Google rejects a sensitive scope that is not on the client's consent screen. Concretely, Thunderbird's Google client is verified for `https://mail.google.com/`, `https://www.googleapis.com/auth/carddav` and `https://www.googleapis.com/auth/calendar` and for nothing else; asking it for `https://www.googleapis.com/auth/contacts` (the People API scope, which is not what Google's CardDAV endpoint wants anyway) fails at consent. The dependency runs the other way round from the prompt order: what can be requested is a property of the application.

Two smaller things fall out of the same setup. The discovery deadline of 8 seconds is longer than an interactive prompt should ever sit still. And the custom application prompt reads `Client id (leave empty for now):`, which suggests the wizard will come back to it; it never does, the value is simply left blank in the fragment for the user to fill in by hand, and the JSON output drops the key entirely instead of carrying the same placeholder.

## What

1. **Fold endpoint spellings of one authorization server into one entry.** Grant reduction compares the flow and the host of the endpoint that starts it (authorization, or device authorization) instead of the full endpoint URLs. Compose yields mechanism outputs in priority order, so the first spelling seen wins its endpoints, which is the fixed provider rule's. Two genuinely different authorization servers still reach the pick list as two entries.
2. **Do not prompt for a single grant.** When reduction leaves one entry, it is carried through without a pick list, as the application step already does.
3. **Resolve metadata into every grant it advertises.** Authorization code when an authorization endpoint is published, device authorization when a device authorization endpoint is, both when both are. The pick list is where that choice belongs, not the resolution.
4. **Ask the application before the scopes.** The application step decides where the scope options come from:
   - a well-known public application offers exactly the scopes it is registered for, with the discovered ones selected;
   - dynamic registration prompts from the authorization server's advertised scopes before it registers, since the scopes travel in the registration request;
   - the custom entry keeps its free-text prompt, seeded with the discovered scopes, since the wizard cannot know what the user registered.
5. **Shorten the discovery deadline** from 8 to 6 seconds.
6. **Say `Client id:`**, with no parenthetical, and carry the key in the JSON output even when it was left empty, matching the `client-id = ""` placeholder the TOML fragment already emits.

## Non-goals

- Changing io-pim-discovery. The duplicate is created by merging, but choosing between merged candidates is the wizard's job, and the collector's merge is what lets one grant cover several services.
- Making discovery sequential, or reporting per-mechanism progress. The fan-out returns outputs in mechanism-priority order already, so a slow mechanism cannot outrank an authoritative one; sequencing would only trade the current bounded wait for a sum of timeouts.
- Validating a scope against the provider. The known applications' registered sets are hardcoded, as Thunderbird hardcodes its own; nothing in OAuth exposes the scopes tied to a client registration.
