---
cairn: log
change: providers-in-the-sample
landed: 2026-11-10
---

# Providers in the sample

config.samples/ is gone. config.sample.toml keeps its `[accounts.example]` table of every field and value, its endpoints block back (the owner's draft had dropped it with `device-authorization` and the three redirections), then `# Providers: uncomment yours in place of the lines above of the same keys.` and three sections of commented code: Google (Pimalaya's client and secret, the latter copied from google.toml by perl, `extras.access_type = "offline"`, the endpoints, the Gmail, Calendar and People scopes, CalDAV and CardDAV commented for an application of one's own), Microsoft (Pimalaya's client, the endpoints, `http://localhost`, the Graph scopes, Outlook IMAP and SMTP commented in their place, where the draft had listed them beside Graph) and Fastmail (`client-id = ""`, the endpoints, `org.pimalaya.ortie://redirect`, the JMAP session resource, the scopes).

The owner then trimmed the sections: no line introducing them, no comment heading a scope group, every scope active, Google's CalDAV and CardDAV and Microsoft's Outlook IMAP and SMTP included. A `# Custom` section closes the file for an application of one's own at any other provider: client id and secret, the four endpoints, the redirection and the scopes, with example values. The owner moved `client-id` and the endpoints block out of the shared fields into the sections, so the file holds an account once one section is uncommented.

The draft's commented `#scopes = [` failed the variant test, which uncommented one line at a time. `sections` (src/config.rs) now finds the paragraphs of commented lines holding code; `sections` requires a heading, the shared client block being commented code only. `variants` uncomments each section whole through `swap`, which keeps prose lines (the heading, `#` spacers) as comments and comments out the active lines of the fields it sets, an array value up to its bracket, so the line numbers hold. On that base it uncomments in turn each commented element inside the section and each line of commented code outside the sections. `key` counts a secret's `raw` and `command` as one field, which the Google base needed (its `client-secret.raw` against the shared `client-secret.command`). `the_sample_configuration_parses_with_each_variant` replaces `every_sample_holds_its_provider_account` and `the_sample_configuration_holds_one_example_account`, and requires the Google, Microsoft, Fastmail and Custom sections to be found; it failed on `#extra.access_type` in the Google section, on `#pkce = "plane"` (as a Google variant) and on an unclosed Custom `scopes`.

The README feature reads "Ready-made providers", its Configuration points at the sections, its flow downloads config.sample.toml with `curl --create-dirs -o`, edits it and runs the commands without `-a`, as do the REPL and MCP examples. The missing-configuration error and the welcome (welcome-without-config) point at config.sample.toml. The CHANGELOG's config.samples entry became the sections' one; the provider-quirks spec and the live-provider-tests proposal name the sections.

Capabilities moved: config (cairn/spec/config.md: "Samples, one per provider" replaced by "One sample configuration, with a section per provider", a missing configuration pointing at config.sample.toml), provider-quirks (the sections apply the quirks).

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`.
