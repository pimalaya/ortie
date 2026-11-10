---
cairn: change
id: providers-in-the-sample
status: landed
created: 2026-11-10
---

# Providers in the sample

## Why

config.samples/ held one ready-made account per provider beside config.sample.toml, so a user starting from the field reference had nothing pointing at the values their provider needs, and the welcome had to pick one provider file. The owner asked on 2026-11-10 for one file to download, sending users to the right provider for the scopes, which is where keeping everything in config.sample.toml comes from; they started Google, Microsoft and Fastmail sections at its end, and chose to delete config.samples/.

## What

- config.sample.toml keeps every field with its values, the endpoints block (device authorization and the three redirections) included, then a line saying a provider section is uncommented in place of the lines of the same keys, then one section per provider: Google (Pimalaya's client and secret, `extras.access_type`, the endpoints, the Gmail, Calendar and People scopes, CalDAV and CardDAV commented for an application of one's own), Microsoft (Pimalaya's client, the endpoints, `http://localhost`, the Graph scopes, Outlook IMAP and SMTP commented in their place) and Fastmail (an application of one's own, the endpoints, the private-use redirection, the JMAP session resource, the scopes). A custom section closes the file for an application of one's own at any other provider (owner, 2026-11-10). The owner then moved `client-id` and the endpoints out of the shared fields into the sections, so the file holds an account once one section is uncommented. Each section is a paragraph of commented code with its heading.
- config.samples/ is deleted.
- The sample test treats a paragraph of commented lines only, opened by a heading, as a section, and uses each one uncommented as a base: on top of it, each commented element inside it and each line of commented code outside the sections is uncommented in turn, in place of the active lines of its field.
- The README, the welcome, the missing-configuration error, the CHANGELOG and the src/main.rs header point at config.sample.toml; the README flow downloads it, and the REPL and MCP examples drop `-a google`.
