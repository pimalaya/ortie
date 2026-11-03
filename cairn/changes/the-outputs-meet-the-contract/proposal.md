---
cairn: change
id: the-outputs-meet-the-contract
status: landed
created: 2026-11-03
---

# The outputs meet the contract

## Why

pimalaya-cli 0.3 (cairn/spec/output.md of the cli repo) settles two rules Ortie still breaks, and himalaya, calendula and cardamum already follow them:

- "Shared nouns keep one shape": the wizard's output is `{name, default, document}`. Ortie's `configure --json` prints the account as a structured object (`clientId`, `endpoints`, `storage`...), and through `skip_serializing_if` drops `default`, `clientSecret`, `grant`, `scopes`, `extras`, `storage` and every endpoint when absent, which also breaks "Absent is null".
- "Values and times have one spelling": a binary's own types print no epoch numbers, an instant it resolves is RFC 3339 in UTC with `Z`. `token inspect` prints `issuedAt` as Unix epoch seconds, a stamp Ortie itself writes (token spec, "issued_at at receipt").

Both are breaking under `--json`, and the release they would ship in is already a major (camelCase keys, receipts), so they should go in the same one rather than force a 4.0.

## What

- `configure` prints `ConfigureOutput { name, default, document }`, `document` being the TOML fragment the text form prints today, the text form unchanged. The structured struct the wizard fills becomes an internal draft type (not printed, no `Serialize`/`JsonSchema`), whose `Display` renders the document.
- `token inspect` prints `issuedAt` as an RFC 3339 UTC string (`humantime::format_rfc3339_seconds`, already a dependency), `null` when unknown. `expiresIn` stays integer seconds. The stored token JSON keeps its epoch `issued_at`: that is the storage format, not output.
- Spec output.md: "JSON keys are camelCase" loses its `ConfigureOutput` sentence; a requirement names the two shapes. CHANGELOG: one **BREAKING** bullet each; the camelCase bullet drops the `configure` keys it lists.

Out of scope: failure codes (`ErrorReport` stays uncoded, `code: null`, which the error spec allows); `token inspect` `scope` stays the server's space-delimited string.
