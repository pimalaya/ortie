---
cairn: log
change: samples-without-comments
landed: 2026-11-09
---

# Samples without comments

config.sample.toml is one `[accounts.example]` table listing every field and each of its values with dotted keys, blank lines grouping the account, the grant, the client, the request, TLS, the endpoints, the storage and the hooks. The default value stands active and the others commented as `#key = value`: `grant`, `pkce` (`true` included), `tls`, `scopes`, `resources`, `alpn` and `endpoints.redirection` (port-less loopback, fixed port, private-use scheme) show every value; a field without a default (`client-secret` raw, shell string and argv, `client-key`, `client-cert`, `extras`, the endpoints but the redirection, the hooks) is commented whole. Every line ends in one sentence after tabs, aligned per block at column 40 or the next tab stop past the block's longest line; the hook sentences name the variables they receive. Commands are `exit 127` placeholders, the shell form, since an argv `["exit", "127"]` would fail to spawn rather than exit 127.

samples/ is renamed config.samples/, beside config.sample.toml, and holds google.toml, microsoft.toml and fastmail.toml, each an `[accounts.<provider>]` table storing its token under `ortie/<provider>`, with no nested table. Google and Microsoft take Pimalaya's application on their active lines, an application of one's own standing as a commented `client-id` (and `client-secret.raw` at Google) under a `# your own application` line. Scopes come in groups under one comment line each: Gmail, the Calendar API and the People API active at Google, CalDAV and CardDAV commented for an own application only; Graph mail, calendar and contacts active at Microsoft, Outlook IMAP and SMTP commented to replace them, Microsoft issuing a token for one resource only. Fastmail has its own-application account only. `check_sample` (src/config.rs) requires the single `[accounts.<name>]` header and parses the file through the loader `-c` uses, then once per entry of `variants`, which uncomments one line of commented code (`#key`, `# key` or a `# "scope"` element) in place of the active line of its key; a misspelled commented `client-id` in microsoft.toml and a `#pkce = "plane"` in config.sample.toml each failed it. `every_sample_holds_its_provider_account` runs it over config.samples/, the name taken from the file, and `the_sample_configuration_holds_one_example_account` over config.sample.toml. The README took over the storage and hook commands to reach for and the registration of an application of one's own, Fastmail's RFC 7591 request included.

Capabilities moved: config (cairn/spec/config.md: "Samples, one per use case" replaced by "Samples, one per provider").

Verified: `cargo fmt`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test --all-features`.
