---
cairn: log
change: the-flow-hardened
landed: 2026-11-19
---

# The authorization flows hardened

Findings ORT-4 (in part), ORT-5, ORT-6 and ORT-7 of the CLI family analysis of 2026-11-15.

- src/account.rs `Account::redirection` returns the listener it bound for an ephemeral port, and src/auth/get.rs waits on it (`await_redirect_on`), binding a configured port only when none was bound. The interactive text of the authorization request prints the redirect URI and the link, no longer the state nor the verifier.
- src/auth/get.rs `is_loopback_redirect` accepts `http` only, which the CLI wait, `auth listen` and MCP `auth_get` share.
- src/mcp.rs `auth_resume` parses the URI and checks its state against the pending flow before taking it.
- src/auth/resume.rs `PkceCodeVerifierParser`, a clap `TypedValueParser` raising its own `ValueValidation` error, used by `auth resume` and `auth listen`; the device handoff prints `auth resume -- '<code>'`.

Evidence, each failing on 58de3a9: tests/auth_listen.rs `an_https_loopback_redirect_is_not_listened_for`, tests/mcp.rs `an_https_loopback_redirect_is_not_captured` and `a_mis_pasted_uri_leaves_the_flow_pending`, tests/exit.rs `a_refused_pkce_verifier_is_not_echoed`, tests/device_grant.rs `the_printed_resume_takes_a_code_beginning_with_a_dash`, and the unit test `a_refused_verifier_is_not_echoed`. The two interactive changes need a terminal and have no test.

Left: the handoff off the command line, proposal the-handoff-leaves-the-command-line.

Capabilities moved: auth ("Authorization code grant", "Manual resume fallback", "Device grant runs", "Loopback listen"), mcp ("The flow lives in the server").
