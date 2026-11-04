---
cairn: log
change: auth-listen
date: 2026-11-04
---

# `auth listen`, and the handoff carrying its redirect URI

A non-interactive or `--json` `auth get` dropped the redirect URI it generated: neither the handoff object nor the printed `auth resume` command carried it, so the code was exchanged without the `redirect_uri` its authorization request had, which RFC 6749 section 4.1.3 requires. `AuthorizationUri` now serializes `redirectUri`, and `print_handoff` (formerly `print_manual_resume`, src/auth/get.rs) puts `--redirect-uri=` on every printed command, preceded by the `auth listen` command when the redirection is loopback.

`auth listen` (src/auth/listen.rs) binds the loopback redirect URI, waits for one redirect through io-oauth's `await_redirect`, then runs `AuthResumeCommand` with it. It is refused on the device and client credentials grants and on a redirection that is not loopback http(s), and is part of the REPL tree.

Capabilities moved: `auth` (Manual resume fallback modified, Loopback listen added).

Verified: `a_handed_off_flow_carries_its_redirect_uri` in tests/auth_listen.rs fails against the previous handoff (no `redirectUri`) and passes now; `a_handed_off_flow_is_listened_for_and_stored` runs `auth get --json`, then `auth listen` with the handed-off values, the test sending the redirect as the browser, and checks the token request carried the code, the redirect URI and the verifier.
