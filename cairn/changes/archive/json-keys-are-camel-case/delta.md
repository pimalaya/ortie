---
cairn: delta
change: json-keys-are-camel-case
---

## ADDED Requirements

### Requirement: JSON keys are camelCase
Every output type Ortie prints SHALL serialize its keys as camelCase, so each key stays reachable by dot access in jq and JavaScript and so the family reads as one surface to a script. A field carrying a provider spelling verbatim SHALL keep its explicit `#[serde(rename)]`, since `@odata.nextLink` and `nextPageToken` are the provider's names rather than derivable ones. Configuration types SHALL stay kebab-case, including the TOML fragment `ConfigureOutput` renders as text: what the loader reads is not what the printer emits.
