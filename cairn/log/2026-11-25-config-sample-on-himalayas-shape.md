---
cairn: log
change: config-sample-on-himalayas-shape
landed: 2026-11-25
---

# The sample on Himalaya's shape

config.sample.toml follows himalaya/config.sample.toml, the model the maintainer chose, and GUIDELINES inline-007.

## What landed

- One commented `#key = value` line per key with a short tab-aligned note; `grant`, `pkce`, `tls.provider` and `tls.rustls.crypto` list their other values in brackets at the end of the note instead of repeating the key on more lines.
- `grant`, `pkce` and `auto-refresh` are commented at their default; `default`, `client-id` and the storage commands, the keys an account needs to load, stay active, so the sample loads as written.
- `tls.client` is written as the flat keys `tls.client.cert` and `tls.client.key`.
- The custom section is folded into the keys: the client, the grant options and the endpoints each carry a note, the README pointer heading the client keys on one line.
- src/config.rs: `the_sample_configuration_parses_with_each_variant` is Himalaya's, ported with its helpers (`SampleLine`, `sample_paragraphs`, `sample_ranks`, `sample_variant`, `load_sample`): the sample loads as written, then once per rank fully uncommented, and the four grants take their turn in order. The `sample` feature of the pimalaya-config dev-dependency is no longer used and the dev-dependency is gone.

## Spec

cairn/spec/config.md: "One sample configuration, the provider accounts in the README" describes the new shape and the new test.
