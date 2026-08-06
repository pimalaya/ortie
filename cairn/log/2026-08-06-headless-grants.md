---
cairn: log
change: headless-grants
landed: 2026-08-06
---

# Headless client credentials grants (RFC 6749 section 4.4, RFC 7523 section 2.2)

Landed the two headless grant kinds so OAuth-ignorant tools exec `ortie token show --auto-refresh` for a guaranteed-fresh bearer token. `GrantConfig` gained `ClientCredentials` and `ClientCredentialsJwt` plus an `is_client_credentials` predicate; `AccountConfig`/`Account` gained `client-key` and `client-certificate` (shell-expanded paths). `auth get` dispatches to a new headless path in auth/get.rs: the plain kind rides Basic auth from the existing `client-secret`, the JWT kind mints a fresh assertion per request (key re-read from disk, PEM-or-DER certificate decoded to DER for a recomputed `x5t`, unique `jti` from the CSRF state random source re-encoded URL-safe, `iss`/`sub` the client id, `aud` the token endpoint, 10 minute validity) through io-oauth `sign_jwt_bearer_assertion` and `request_jwt_bearer_client_credentials`; the client secret stays unset on that path. `auth resume` rejects both kinds.

The refresh decision moved behind `refresh_action` in token/refresh.rs: `Reacquire` on the client credentials kinds (a re-run of the grant, persisting and firing the on-refresh hooks via the new `TokenRefreshCommand::reacquire`), `Refresh` where a refresh token exists, `Keep` otherwise. `token show` follows it, and on an auto-refreshing client credentials account a missing or unreadable stored token re-acquires instead of failing, so the first run needs no prior `auth get`. An `invalid_client` on the JWT kind appends a certificate renewal hint to the reported error chain.

io-oauth comes from git master via a `[patch.crates-io]` (crates.io 0.2.0 predates rfc7523) with the `jwt-bearer` feature enabled. Tests: config parsing for both kinds and the `refresh_action` branch as unit tests; a tests/client_credentials.rs mock-endpoint suite covering expired-token re-acquisition, empty-storage acquisition, per-mint assertion freshness (distinct `jti`, `x5t` thumbprint, no Basic header) and the renewal hint.

Spec: `config` (MODIFIED flat grant selector; ADDED JWT assertion credentials), `auth` (ADDED client credentials grants, certificate renewal hint), `token` (ADDED auto-refresh branches per grant). Deferred, per the proposal: the RFC 7523 section 2.1 authorization grant (Google service accounts, different claim semantics) and command-sourced private keys (the secret command resolver is single-line, PEM is not).
