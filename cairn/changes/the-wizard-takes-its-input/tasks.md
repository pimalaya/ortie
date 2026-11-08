---
cairn: tasks
change: the-wizard-takes-its-input
---

# Tasks

- [x] `ConfigureCommand` holds `[INPUT]`, `--name` and `--service`; the first-run offer runs it with none
- [x] The positional skips the address prompt, then trims, validates and discovers as a typed answer
- [x] `--service` parsed from the pick list's wire names; `search::retain_services` filters the grants, typed-URL grants kept
- [x] A filter leaving no grant stops on the hand-written sample, naming the input and the services
- [x] `--name` taken verbatim; a held name asks to keep it, kept prints `{name, default, document: null}`, declined fails naming it
- [x] `ConfigureOutput.document` nullable; schema and registry check pass
- [x] Tests: the hand-off invocation parses, unknown services refused, the service filter, the held name
- [x] fmt, clippy, test, `--no-default-features --features rustls-ring` check (bare `--no-default-features` fails on HEAD already), debug `configure --help`
- [x] README, CHANGELOG
- [x] Fold into cairn/spec/discovery.md, cairn/spec/config.md and cairn/spec/output.md; log; land
