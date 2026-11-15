---
cairn: delta
change: the-live-suites-run-weekly
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Live provider suites
(packaging) The binary SHALL carry one ignored integration suite per provider under tests/ (google.rs, microsoft.rs, fastmail.rs), run with `--ignored` and sharing tests/common.rs with the mock suites. Each SHALL hold an `anonymous` module that runs on public or made-up clients and touches no account, its accounts taken from the provider sections of config.sample.toml where one fits so a provider drifting from the sample fails it, and MAY hold a `credentialed` module reading `ORTIE_`-prefixed variables, which CI runs in the upstream repository only. CI SHALL run the suites weekly (Mondays 02:00 UTC, the family's slot for ortie) and by hand (`gh workflow run tests.yml`), never on a push, and a release SHALL run them by hand on the commit it tags; the offline jobs run on every push. A client secret SHALL reach the binary through a secret command and a key or certificate through a file only its owner reads, never argv nor the configuration, and no assertion SHALL print a token or secret. CI SHALL run each suite in its own job at `RUST_LOG=debug` at most.

## REMOVED Requirements

None.
