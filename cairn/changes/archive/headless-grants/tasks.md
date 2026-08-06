---
cairn: tasks
change: headless-grants
---

- [x] Depend on io-oauth git master with the `jwt-bearer` feature (crates.io 0.2.0 lacks rfc7523)
- [x] Add the `client-credentials` and `client-credentials-jwt` values to the flat `grant` selector
- [x] Add the `client-key` and `client-certificate` account config fields and their `Account` counterparts
- [x] Run both exchanges from `auth get` (secret through Basic auth, assertion minted fresh per request with `x5t` from the certificate)
- [x] Reject `auth resume` on client credentials accounts
- [x] Branch auto-refresh per grant: refresh-token POST where one exists, silent re-acquisition for both client credentials kinds (`token show --auto-refresh` and `token refresh`)
- [x] Re-acquire instead of failing when the stored token is missing or unreadable on an auto-refreshing client credentials account
- [x] Hint at certificate renewal on `invalid_client` for the JWT kind
- [x] Update config.sample.toml (Microsoft-shaped examples), README and CHANGELOG
- [x] Unit tests: config parsing for both kinds, the refresh-decision branch
- [x] Integration tests against a scripted local token endpoint: re-acquisition end to end for both kinds (fixture RSA key), fresh-assertion uniqueness, the invalid_client hint
