---
cairn: delta
change: the-outputs-meet-the-contract
---

## ADDED Requirements

### Requirement: Shared shapes and spellings
`configure` SHALL print `{name, default, document}`, the account name, whether it claims the default, and the TOML fragment its text form prints, every key present. `token inspect` SHALL print `issuedAt` as an RFC 3339 instant in UTC with `Z`, `null` when unknown, and no output type SHALL print an epoch number.

## MODIFIED Requirements

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`. Configuration types SHALL stay kebab-case, including the TOML document `configure` prints: what the loader reads is not what the printer emits.
