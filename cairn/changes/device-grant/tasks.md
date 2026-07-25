---
cairn: tasks
change: device-grant
---

- [ ] Add `endpoints.device-authorization` to the config DTO and its `Account` counterpart
- [ ] Dispatch `auth get` on the configured grant; run the device path (device authorization request, user-code display, polling loop, storage write, on-issue hooks)
- [ ] Remove the `grant = "device"` bail placeholders in `auth get` / `auth resume`
- [ ] Interpret `auth resume` positional per grant (redirected URI vs device code)
- [ ] Reject `--state` / `--pkce` / `--redirect-uri` on device accounts
- [ ] Update `config.sample.toml`, README and CHANGELOG
