---
cairn: change
id: welcome-without-config
status: landed
created: 2026-11-10
---

# Welcome without config

## Why

The welcome left with the wizard (samples-replace-the-wizard): a first `ortie` printed the help, and a command without a configuration failed on one line pointing at the samples. The owner asked on 2026-11-10 for the welcome back on a first run or a missing configuration, default or given, explaining how to get a working one.

## What

- A missing configuration is a typed error, `ConfigMissingError` (src/cli.rs), carrying the path account resolution looked for. A bare `ortie` without a configuration returns it too, the help staying for a configuration that exists or fails to parse.
- `main` meets it with the welcome on stderr and exit 1 when stderr is a terminal and `--json` is off; a script and a JSON caller keep the error.
- The welcome frames Ortie, names the missing path, gives the `curl --create-dirs` command downloading config.sample.toml there, invites the reader to edit it and uncomment the options they need, and gives `auth get` with the `-c` paths. Paths are single quoted, so each command runs as printed. The owner asked the same day for the welcome to download config.sample.toml rather than one provider sample, the providers being folded into it (providers-in-the-sample), then for a shorter welcome: the paragraph on what Ortie does went, and the one walking the provider sections and the storage commands became one sentence.
