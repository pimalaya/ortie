---
cairn: change
id: first-run-wizard-frame
status: landed
created: 2026-08-14
---

# Adopt Comodoro's first-run wizard frame

## Why

Comodoro's `first-run-wizard` landed the shape a Pimalaya CLI meets a newcomer with, and Himalaya has just taken it. Ortie already has the pieces that change what the wizard produces, from `wizard-himalaya-parity` through `wizard-print-then-save`; what it does not have is the frame deciding when the wizard is reached at all.

`take_account` bails with `Config file not found` and `Account not found`. Those are the two messages Comodoro's log named as the thing to fix: neither says which path was read, which accounts exist, nor how to pick one. Someone who mistyped `-c` gets the same four words as someone who has never run the tool.

Nothing offers to fix it either. A missing configuration is a dead end from every command: the wizard exists, and the command that needed an account never mentions it.

There is no way to run the wizard by name. Bare `ortie` runs it, so configuring a second account means running the binary bare, which also means running it bare is never how you see the help. A user who is set up and types `ortie` to remind themselves of the commands gets a wizard instead.

Nothing guards the terminal on the way in. A cron job hitting the wizard gets prompts it cannot answer.

`ConfigPathsArg` reads no environment variable and does not split on `:`, both of which the other products have.

## What

The wizard's content is untouched. Discovery, the application step, the scopes, the storage and the grant test all stay exactly as they are.

An `ortie configure` command (alias `wizard`) runs the wizard by name, with no welcome, since whoever typed it knows what it does. The welcome belongs to the offer, and gains the configuration path that was looked for.

A bare `ortie` stops running the wizard unconditionally. With no configuration it raises the offer; with one it shows the help, which is what someone already set up is asking for. `--account` with no subcommand is a half-typed command, so it shows the help too.

The offer becomes a hook raised from the two places nothing can happen without a configuration: the bare invocation, and any command needing an account. It never ends the process: a command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail on the configuration it still has not got.

Nothing prompts when stdin is not a terminal or `--json` is set.

The save stops prompting for a path and takes it from `Config::target_path`, which is where `-c` pointed or the default location. The generated account takes a free name, suffixed until it is, and claims `default` only when no other account does.

The three resolution failures each name what is missing and what to do: the path that was read, the accounts the configuration holds, and the two ways to pick a default.

`ORTIE_CONFIG` is read like `-c`, and both split on `:`.

## Scope / non-goals

Print-then-save stays. `wizard-print-then-save` decided the fragment always reaches stdout before the save is offered, because choosing where a configuration goes before having seen it is the wrong order. Comodoro and Himalaya print only as the fallback, when the save is declined or the stream is not a terminal. Ortie's order is the better one and the argument for it is on the record, so the frame adapts to it rather than the reverse: what changes is where the file lands and under what name, not when the fragment is shown.

Appending already works and keeps its confirmation, since the file being appended to is one the user already owns. What it gains is the two invariants Comodoro established, a free account name and a single default, which appending needs and did not have.

No rendering change. Ortie's fragment is one account with a flat set of keys, and `wizard-himalaya-parity` already settled what it looks like.
