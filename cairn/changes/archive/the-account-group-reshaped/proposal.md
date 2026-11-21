---
cairn: change
id: the-account-group-reshaped
status: landed
created: 2026-11-21
---

# The account group reshaped for a binary without backends

## Why

The account group of 2026-11-15 (cairn/log/2026-11-15-account-group.md) printed pimalaya-cli's shared `AccountListOutput` and `AccountCheckOutput`, built for binaries with protocol backends. Ortie has none: `account list` printed `backends: []` and `notBuilt: []` forever, `account check` called `storage` and `endpoints` "backends", lumped every endpoint into one row that stopped at the first failure, and never said whether the stored token is usable. Ortie always runs on one account, so the list of one checked account was noise too. The owner decided the shapes below on 2026-11-21, before the 3.0.0 release; the kit spec gives a binary without backends the room for them (pimalaya-cli cairn/spec/commands.md, "Every binary carries an account group", which also allows `account show`).

## What

1. `account list` (alias `ls`) prints `{"accounts": [{"name", "default", "grant"}]}`, sorted by name, `grant` as the configuration spells it. Text: one aligned line per account, its name, its grant and `default`. It still reaches nothing.
2. `account show` (new) takes the account as every account-bound command does and prints every property of it as the configuration loads it: every `-c` layer merged, every default filled in, an unset optional left out. It reaches nothing. Text: TOML, `[accounts.<name>]` then the keys in their configuration spelling, so it pastes back into a configuration. JSON: `{"account": {"name", ...}}`, camelCase. The one raw secret the configuration types hold, `client-secret.raw` (pimalaya-config's `Secret::Raw`), prints `"<redacted>"` in the text and `{"raw": null, "redacted": true}` in the JSON; commands and paths print as loaded.
3. `account check` prints `{"account", "checks": [{"check", "ok", "code", "error", "tls"}]}`, the rows `storage`, `token`, then one per endpoint the grant uses, named by its configuration key. `token` reads what the storage returned, sending nothing: missing fails coded `token-missing`; expired with no refresh token, on a grant that does not run again by itself, fails coded `token-expired` (new); expired with a way to renew it passes. A failed storage fails the token row too: `token-missing` on an empty store, uncoded otherwise. Each endpoint row is checked on its own: one the grant needs and the account omits fails coded `endpoint-missing`, a configured one gets today's `HEAD`. `tls` rides on `https` endpoint rows only. A failed row exits 3, as before.
4. The three outputs are Ortie's own types, with their schemas `ortie-account-list`, `ortie-account-check` and the new `ortie-account-show`.

## Decisions taken while implementing

- An expired token on a client credentials grant passes the token row: it holds no refresh token by design, and auto-refresh or `token refresh` re-runs the grant. The design's "expired with no refresh token fails" is read as "with no way to renew it".
- The endpoints a grant uses: `authorization`, `token` and `revocation` for `authorization-code`; `device-authorization`, `token` and `revocation` for `device`; `token` and `revocation` for the client credentials grants. `revocation` is never needed, so an unset one prints no row. A configured endpoint the grant has no use for (an `endpoints.authorization` on a device account) is no longer sent a request.
- An endpoint missing no longer stops the requests: the other endpoint rows are still sent theirs.
- `pkce` prints as the configuration writes it, `"s256"`, `"plain"` or `false`, in the JSON too.
- Paths print as loaded, after the shell expansion the loader runs (`~/key.pem` prints expanded). Lists merged across `-c` layers print concatenated, as the loader merges them.
