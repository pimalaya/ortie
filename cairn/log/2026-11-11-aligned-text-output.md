---
cairn: log
change: aligned-text-output
landed: 2026-11-11
---

# Aligned text output

Ortie pins pimalaya-cli 5dd9416, which adds `fields::Fields`. `TokenInspectOutput` (src/token/inspect.rs) renders `Type`, `Issued`, `Expires`, `Refresh token` and `Scopes` through it, the scope string split on whitespace into one line per scope, `Expires` reading `in 2m 34s`, `expired` or `unknown`, and the token type no longer lowercased. `AuthorizationUri` and `DeviceAuthorization` (src/auth/get.rs) drop their `Created ... request with:` bullets for `Fields`, the device expiry through humantime, then print the link after a blank line with a two-space indent; `print_handoff` and the device handoff indent their `auth listen` and `auth resume` commands the same way instead of prefixing `> `.

Seen against the mock server of text-output-ends-with-newline (inspect, the code and device handoffs, piped) and with `token inspect` on the owner's Google test account, its four scopes one per line.

Capabilities moved: output (cairn/spec/output.md: "A text record prints as aligned fields" added).
