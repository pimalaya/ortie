---
cairn: delta
change: samples-without-comments
---

## ADDED Requirements

### Requirement: Samples, one per provider
The repository SHALL ship config.samples/<provider>.toml, each holding one complete `[accounts.<provider>]` table with dotted keys and no nested table, ready to append to a configuration. Its active lines SHALL use Pimalaya's application where one exists; an application of one's own and the other scopes SHALL stand as commented code. One comment line SHALL head each scope group and each alternative, naming what it serves, and no other comment SHALL appear. config.sample.toml SHALL be one `[accounts.example]` table listing every field and each of its values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. The test suite SHALL parse each sample and config.sample.toml through the loader `-c` uses, as it stands and once per line of commented code swapped in for the active line of its key, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN a sample whose commented code sets `client-id`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

## MODIFIED Requirements

## REMOVED Requirements

### Requirement: Samples, one per use case
Replaced by "Samples, one per provider": the samples of one provider differed only by client and scopes, which now stand as commented code in one file.
