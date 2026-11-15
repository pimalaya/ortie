---
cairn: tasks
change: auth-listen
---

# Tasks

- [x] `auth get` handoff: `redirectUri` in the JSON object, `--redirect-uri=` in every printed command (test fails against the previous handoff)
- [x] `AuthListenCommand` (src/auth/listen.rs): flags as `auth resume`, loopback check, `await_redirect`, then the resume exchange
- [x] Refused on the device and client credentials grants
- [x] Human handoff prints the listen command before the resume one
- [x] `auth listen` in the REPL tree
- [x] Tests: auth get then listen then a redirect stores the token with the handed-off verifier and redirect URI; a non-loopback redirect and a device account are refused
- [x] `ortie-auth-listen` in `json-schema`
- [x] README usage, CHANGELOG
- [x] Fold into cairn/spec/auth.md; log; land
