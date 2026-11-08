---
cairn: log
change: the-wizard-takes-its-input
landed: 2026-11-08
---

# The wizard takes its input

`ConfigureCommand` (src/wizard.rs) holds `[INPUT]`, `--name` and `--service`, so the other Pimalaya CLIs can hand OAuth 2.0 off with `ortie configure <ADDRESS> --name <NAME> --service <SERVICE>` then `ortie auth get -a <NAME>`; the first-run offer runs it with none of them. The positional skips the address prompt and is trimmed, validated and discovered as a typed answer.

`--service` parses through `search::service_parser`, a possible-values parser over the wire names `service_name` already labelled the pick list with, so an unknown name is a clap error listing the fourteen valid ones. `search::retain_services` keeps the grants tagged with any named service, and the untagged grants a typed URL resolves to; a filter leaving none stops through `stop_undiscovered`, which now names the services.

`--name` is taken as written. `ExistingConfig` keeps the names claiming the default (`defaults` in place of `has_default`), and `ExistingConfig::keep` answers the output for an exactly held name: the wizard asks to keep it before any other prompt, prints `ConfigureOutput` with `document: null` when kept, and fails naming the account when declined. `ConfigureOutput.document` became `Option<String>`, the `ortie-configure` schema typing it `string` or `null`.

Capabilities moved: discovery (cairn/spec/discovery.md: "Input orients the flow", "Grant-based service reduction", "A single candidate is not a choice", "Account name derived, not prompted", "Stop when nothing is discovered" and "Complete, paste-ready fragment" modified), config (cairn/spec/config.md: "A named command runs the wizard", "A generated account takes a free name and one default" and "The wizard saves where the configuration lives" modified), output (cairn/spec/output.md: "Shared shapes and spellings" modified).

Verified: `the_hand_off_invocation_parses`, `a_bad_service_or_an_empty_name_is_a_usage_error`, `the_service_filter_keeps_the_grants_serving_a_named_service` and `a_given_name_is_kept_only_when_held_as_written`; the registry check passes. Through a pseudo-terminal on a scratch configuration: the kept path printed `{"name": "work", "default": true, "document": null}` and left the file byte-identical, the declined path exited 1 naming the account, and a positional loopback URL ran discovery without the address prompt. `cargo check --no-default-features` fails as it did before this change (io-oauth's `Oauth20ClientStd::connect` needs a TLS feature); `--no-default-features --features rustls-ring`, the build without the wizard, is clean.
