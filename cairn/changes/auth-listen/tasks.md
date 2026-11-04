---
cairn: tasks
change: auth-listen
---

# Tasks

- [ ] `auth get` handoff: `redirectUri` in the JSON object, `--redirect-uri=` in every printed command (test fails against the current handoff)
- [ ] `AuthListenCommand` (src/auth/listen.rs): flags as `auth resume`, loopback check, `await_redirect`, then the resume exchange
- [ ] Refused on the device and client credentials grants
- [ ] Human handoff prints the listen command before the resume one
- [ ] `auth listen` in the REPL tree
- [ ] Tests over a mock token endpoint: listen then a redirect stores the token; a non-loopback redirect is refused; a device account is refused
- [ ] `ortie-auth-listen` in `json-schema` if its output differs from the resume receipt's entry
- [ ] README usage, config.sample.toml note if any, CHANGELOG
- [ ] Fold into cairn/spec/auth.md; log; land
