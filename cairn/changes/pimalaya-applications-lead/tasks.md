---
cairn: tasks
change: pimalaya-applications-lead
---

# Tasks

- [x] `KNOWN_APPS`: Pimalaya's Google Desktop client and Microsoft Entra application ahead of Thunderbird's, the Google secret split in two
- [x] `KnownApp.services`, `known_apps` matching the grant's services; `AccountDraft.services` carried from the chosen grant
- [x] `KnownApp.narrows`, applied to the discovered scopes when a known application is picked
- [x] `search` composes the `--service` services when named, the protocol set otherwise
- [x] A port-less `http` loopback redirection binds an ephemeral port in `auth get` and in the MCP server
- [x] Tests: Pimalaya leads on Google and Microsoft, the service match, the calendar narrowing, the ephemeral loopback port; the composed services checked under a pseudo-terminal (discovery is network-bound)
- [x] Spec folded (discovery, auth, provider-quirks), log, CHANGELOG, README
- [x] fmt, clippy, test, `--no-default-features --features rustls-ring` check
- [ ] Live, by the owner: a Google and a Microsoft sign-in through `ortie configure` then `auth get` with the Pimalaya clients, on the test accounts
