---
cairn: log
change: wizard-default-command
landed: 2026-07-13
---

# Wizard as default command with a complete fragment

Bare `ortie` now runs the discovery wizard (the root parser's subcommand became optional); `ortie auth discover` remains the explicit spelling. The fragment is complete and appendable per D6: pure valid TOML on stdout with guidance as leading comments, an `[accounts.<name>]` header (name prompted, input domain or host suggested, quoted when not a bare key), an optional client-id prompt (commented placeholder when empty), a commented `default = true`, and a pass-style storage template. Prompts render on stderr (inquire) and the no-service narration moved off stdout, so `ortie >> <config>` appends cleanly. README (Features, Configuration, Usage) and `config.sample.toml` updated; the previously open which-path-to-write question is void.

Spec: `discovery` (wizard is the default command, prints never writes, complete paste-ready fragment).
