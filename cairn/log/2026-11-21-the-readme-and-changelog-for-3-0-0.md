---
cairn: log
change: the-readme-and-changelog-for-3-0-0
landed: 2026-11-21
---

# The README and the CHANGELOG for 3.0.0

The README and the CHANGELOG were aligned on the `markdown`, `readme` and `changelog` scopes of the Pimalaya guidelines, on himalaya's and neverest's shape, ahead of 3.0.0.

## What landed

- README: eight Features bullets of at most 120 characters, the OAuth 2.1 list folded into the RFC coverage table (RFC 8252 and RFC 9700 rows added), the installation entries in `<details>` blocks, Sources building with cargo. Configuration keeps the welcome, the three paths and `-c`, and gains Google, Microsoft and Fastmail subsections carrying the caveats of the provider-quirks spec and pointing at the sample's sections. Usage is one paragraph and one shell block; the exit codes, the failure object, the REPL and MCP subsections and the MCP client JSON leave for `--help`. The Alternatives section and the cargo-feature tip go, readme-007 and readme-017 having no place for them.
- CHANGELOG [Unreleased]: dependency bumps and swaps without a user-visible effect dropped (pimalaya-cli 0.8.0, jiff for humantime, base64-codec, pimalaya-config 0.5.2, the removed dependencies), the io-oauth bump kept as its wire change, duplicates merged (the device bounds, the REPL grammar), the library-driven fixes split one per behaviour, the schema list completed, `auth get` no longer opening a browser and the REPL line splitting marked breaking. Released sections: the two 0.1.0 summaries not in the past tense reworded.
- Cargo.toml `description` already matched the README sentence (cargo-002); unchanged.

## Open

- readme-011 wants each provider's ready-made block in the README and config.sample.toml reduced to the skeleton; the blocks stay in the sample, which the variant test and the live suites read, and no client id was copied into the README.
- readme-006 terminal block: not applied, Ortie having no logo.svg; a tests/docs.rs would be needed first.
- The family-wide Sponsor badge in the header is outside the readme-004 template.

## Capabilities moved

None: documentation only.
