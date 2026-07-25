---
cairn: log
change: dynamic-client-registration
landed: 2026-07-15
---

# M5: dynamic client registration in the wizard

Landed ahead of the device grant and discovery upgrades, since nothing in it depended on them. The application step now offers every way to obtain a client, sorted by a new io-oauth preference order: dynamic registration when the provider advertises it, well-known public applications registered against the same authorization server, then the custom entry. Because no discovery mechanism hands the wizard registration support, it probes at the application step (issuer guessed as `https://<host>` from each endpoint, metadata fetched via `ComposeClientStd::oauth_server`, the entry hidden without a `registration_endpoint`). Google and Microsoft publish none, so their lists keep leading with Thunderbird; Fastmail advertises one.

io-oauth grew `rfc7591::source::Oauth20ClientSource` and `Oauth20ClientStd::register_client` (the std client moved to the crate-root client module); Ortie path-patches io-oauth until the next release. Registration runs at wizard time keeping the print-only philosophy: the issued client id (and client secret as the config secret shape) land in the fragment. Confirmed against the live endpoint, Fastmail's registration rejects every http/https redirect and accepts only a reverse-DNS private-use scheme (RFC 8252 §7.1), so the wizard registers `http://127.0.0.1` first and retries with `org.pimalaya.ortie://redirect`, pinning `endpoints.redirection`; `auth get` then detects the non-loopback redirection, skips the listener, and prints the manual `auth resume`. A failed registration falls back to the remaining pick-list entries.

Spec: `discovery` (client source preference order, registration honours print-only), `auth` (manual resume fallback), `provider-quirks` (Fastmail resource and redirect).
