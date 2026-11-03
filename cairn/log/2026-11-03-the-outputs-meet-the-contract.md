---
cairn: log
change: the-outputs-meet-the-contract
date: 2026-11-03
---

# The outputs meet the contract

`configure` prints `ConfigureOutput { name, default, document }` (src/wizard.rs), the shape himalaya, calendula and cardamum print, `document` being the TOML fragment its text form already printed, so the text output is unchanged. The struct the wizard fills became `AccountDraft`, no longer `Serialize` nor `JsonSchema`, its `Display` the fragment; `config::is_default` went with its last `skip_serializing_if`. `token inspect` serializes `issuedAt` through `serialize_rfc3339` (src/token/inspect.rs) as an RFC 3339 UTC instant, the stored token keeping its epoch `issued_at`.

Capabilities moved: output (cairn/spec/output.md: "Shared shapes and spellings" added, "JSON keys are camelCase" modified).

Verified: `cargo clippy --all-targets` clean with all features and with `rustls-ring` alone, `cargo test --all-features` passed, its inspect test asserting `"issuedAt":"2023-11-14T22:13:20Z"`; `json-schema --dir` describes `ortie-configure` as the three required keys and `ortie-token-inspect` `issuedAt` as a nullable string.
