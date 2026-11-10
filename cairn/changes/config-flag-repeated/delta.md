---
cairn: delta
change: config-flag-repeated
---

## ADDED Requirements

### Requirement: The configuration path comes from `-c` only
The configuration path SHALL be overridden by `-c` only, repeatable, the first path the base and the next ones merged on top in order. Each occurrence SHALL be one path, never split on a delimiter, and no environment variable SHALL be read for it.

#### Scenario: A Windows path is read whole
- GIVEN `-c C:\Users\me\config.toml`
- WHEN the configuration is loaded
- THEN it is read from that one path, not from `C` and `\Users\me\config.toml`

## MODIFIED Requirements

### Requirement: Manual resume fallback
The printed commands, and the one the device grant prints when it hands off, SHALL carry one `-c` per configuration path and the `-a` name `auth get` ran with, each single quoted, and omit either flag when it was not given. (Other paragraphs unchanged.)

## REMOVED Requirements

### Requirement: The configuration path is read from the environment
Replaced by "The configuration path comes from `-c` only": `ORTIE_CONFIG` is gone and `-c` no longer splits on `:`.
