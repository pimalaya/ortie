---
cairn: change
id: samples-without-comments
status: landed
created: 2026-11-09
---

# Samples without comments

## Why

config.sample.toml had grown into 180 lines of comments around a few fields, and every sample opened on a paragraph of its own. The owner asked on 2026-11-09 for something short: fields grouped by section, a single comment line only where a value is not obvious, and the samples grouped by provider with no comment at all.

## What

- config.sample.toml: one `[accounts.example]` table listing every field and each of its values with dotted keys, blank lines grouping them. The default value stands active and the others commented as `#key = value`, a field without a default (the client secret, key and certificate, the endpoints but the redirection, the hooks) commented whole; every line ends in one tab-aligned sentence, the hooks' naming the variables they receive. Commands are `exit 127` placeholders rather than a real store (owner, 2026-11-09).
- samples/ becomes config.samples/ (owner, 2026-11-09), one file per provider (google.toml, microsoft.toml, fastmail.toml), each an `[accounts.<provider>]` table storing its token under `ortie/<provider>`, with no nested table (owner, 2026-11-09). The samples of one provider differed only by client id, secret and scopes, so Pimalaya's application takes the active lines and an application of one's own and the other scopes stand as commented code (owner, 2026-11-09). One comment line heads each scope group and each alternative; Outlook IMAP and SMTP replace the Graph scopes rather than join them, Microsoft issuing a token for one resource only.
- The test suite checks every sample and config.sample.toml: a single header naming the provider (`example` for config.sample.toml), parsed through the loader `-c` uses, then once per line of commented code swapped in for the active line of its key.
- The README carries what the comments said that a user cannot guess: which commands storage and hooks can run, and how to register an application of one's own at Google, Microsoft and Fastmail, Fastmail's registration request included.
