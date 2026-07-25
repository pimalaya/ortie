---
cairn: log
change: remove-auth-discover
landed: 2026-07-25
---

# Remove the `auth discover` subcommand

Dropped `ortie auth discover`: it ran the exact same `AuthDiscoverCommand::execute` as bare `ortie`, so it was redundant. The only thing it added was a positional input seed (`ortie auth discover me@example.com`), judged not worth a duplicate command; bare `ortie` prompts for the input interactively. Removed the `Discover` variant from `AuthCommand`, dropped the now-unreachable `input` field on `AuthDiscoverCommand` (and its `clap::Parser` derive, so it is a plain wizard entry rather than a command), and made bare `ortie` construct it directly. The wizard itself is unchanged.

Spec: `discovery` (MODIFIED "Wizard is the default command": bare `ortie` is the only entry, no `auth discover` spelling).
