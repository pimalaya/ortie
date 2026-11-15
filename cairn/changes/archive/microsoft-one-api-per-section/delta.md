---
cairn: delta
change: microsoft-one-api-per-section
---

## MODIFIED Requirements

### Requirement: One sample configuration, with a section per provider
The repository SHALL ship config.sample.toml, and the release archives and the Nix package SHALL install it as share/doc/ortie/config.sample.toml, which install.sh copies under its prefix. It holds one `[accounts.example]` table with dotted keys and no nested table. It SHALL list the fields every provider shares and each of their values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. It SHALL end with one section per provider (Google, Microsoft, Fastmail), Microsoft having one for Graph and one for IMAP and SMTP since Entra issues a token for one resource, and a custom one for an application of one's own at any other provider, each a paragraph of commented code only, headed by its name, holding the client, the endpoints, the redirection, the resources and the scopes the provider needs, Pimalaya's application where one exists and example values in the custom one. The file SHALL hold a working account once one section is uncommented. The test suite SHALL parse config.sample.toml through the loader `-c` uses with each section uncommented whole, then on top of it each commented element inside it and each line of commented code outside the sections, uncommented in place of the active lines of its field, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose commented code sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample
