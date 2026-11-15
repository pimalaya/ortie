---
cairn: tasks
change: providers-in-the-sample
---

- [x] config.sample.toml: the owner's sections completed from config.samples/ (the Google secret copied by perl, never printed), Microsoft's Outlook scopes moved to commented alternatives, the generic endpoints block restored
- [x] The owner then dropped the scope group comments and the line introducing the sections, every scope listed active
- [x] Custom section for an application of one's own, every section heading checked by the test
- [x] The owner moved `client-id` and the endpoints into the sections: the test rebased on each section uncommented, a section needing a heading (the shared client block is commented code only), an array value replaced up to its bracket, a secret's `raw` and `command` one field
- [x] config.samples/ deleted
- [x] `sections`, `swap` and `variants` (src/config.rs); `the_sample_configuration_parses_with_each_variant` replaces the two sample tests; seen failing on `#extra.access_type` in the Google section, `#pkce = "plane"` and an unclosed Microsoft `scopes`
- [x] README, welcome and error message, CHANGELOG, src/main.rs header, provider-quirks spec, live-provider-tests proposal
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
