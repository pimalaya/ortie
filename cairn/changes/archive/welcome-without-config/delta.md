---
cairn: delta
change: welcome-without-config
---

## ADDED Requirements

### Requirement: A missing configuration meets a person with the welcome
When no configuration exists at the path `-c` gave or the default one, and stderr is a terminal with `--json` off, a bare `ortie` and every account-bound command SHALL print the welcome on stderr and exit 1, printing nothing more. The welcome SHALL frame Ortie in one sentence, name the missing path, give the `curl --create-dirs` command downloading config.sample.toml there, invite the reader to edit it and uncomment the options they need, and give the `auth get` command carrying the `-c` paths, every path single quoted so each command runs as printed. Otherwise the missing-configuration error SHALL stand, as JSON under `--json`.

#### Scenario: A first run
- GIVEN no file at $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml or $HOME/.ortierc
- WHEN `ortie` runs in a terminal
- THEN the welcome names $XDG_CONFIG_HOME/ortie/config.toml, the command downloading config.sample.toml there and `ortie auth get`, and the exit status is 1

## MODIFIED Requirements

### Requirement: A bare invocation shows the help
A bare `ortie` SHALL show the help when a configuration exists, a broken one included, and so SHALL `--account` with no subcommand, a half-typed command. Without one it SHALL fail as an account-bound command does, the welcome or the error. Nothing prompts: Ortie writes no configuration.

## REMOVED Requirements
