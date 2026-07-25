---
cairn: tasks
change: device-grant
---

- [x] Add `endpoints.device-authorization` to the config DTO and its `Account` counterpart
- [x] Dispatch `auth get` on the configured grant; run the device path (device authorization request, user-code display, polling loop, storage write, on-issue hooks)
- [x] Remove the `grant = "device"` bail placeholders in `auth get` / `auth resume`
- [x] Interpret `auth resume` positional per grant (redirected URI vs device code)
- [x] Reject `--state` / `--pkce` / `--redirect-uri` on device accounts
- [x] Update `config.sample.toml`, README and CHANGELOG
