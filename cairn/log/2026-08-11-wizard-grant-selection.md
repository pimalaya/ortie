---
cairn: log
id: wizard-grant-selection
date: 2026-08-11
---

# Pick one grant per authorization server, and scopes the application can carry

Setting up a Gmail account showed the wizard offering the same authorization server twice, resolving at most one grant per server, and asking for scopes before the application that has to carry them. All three are now fixed, in the wizard only: io-pim-discovery is untouched.

**Grants reduce by authorization server, not by endpoint URL.** `same_grant` compares the flow and the host of the endpoint that starts it, so Google's two descriptions (the fixed provider rules' `accounts.google.com/o/oauth2/v2/auth` with `oauth2.googleapis.com/token`, and Mozilla autoconfig's legacy `accounts.google.com/o/oauth2/auth` with `www.googleapis.com/oauth2/v3/token`, which is also the pair Thunderbird hardcodes) fold into one entry. Compose yields mechanism outputs in priority order, so the surviving spelling is the provider rules', and the legacy pair Google can reject at consent no longer reaches a config. Two genuinely different authorization servers still show up as two entries. The pick-list label lost its `via <token endpoint>` suffix, which only ever existed to tell the duplicates apart, and omits the service list when there is none (a grant resolved from a typed issuer URL).

**A single grant is no longer prompted for.** `configure_discovery` carries a lone entry through, as the application step already did.

**Metadata resolves into every grant it advertises.** `grant_of` became `grants_of`, returning the authorization code grant, the device grant, or both, instead of hiding the device flow behind the redirect one. RFC 8414 section 2 and RFC 8628 section 4 let a server publish both, and a machine with no browser wants the device flow even where a redirect is possible.

**The application step runs first and decides the scope options.** `client::configure` now takes the run's authorization server metadata and returns a `scope::Source`: the registered set of a well-known public application, the scopes the server and provider quirks advertise, or nothing left to ask. A new `src/wizard/scope.rs` holds the step. This is what stops a Google account from being configured with a People API scope: the Thunderbird client id is verified for mail, CardDAV and CalDAV only, and asking it for anything else fails at consent. Dynamic registration prompts from inside its own branch, since the scopes travel in the registration request.

Also: the discovery deadline dropped from 8 to 6 seconds, the custom application prompt reads `Client id:` with no misleading parenthetical, and the client id key is serialized even when empty so JSON carries the same placeholder the TOML fragment does.

Capabilities moved: discovery.
