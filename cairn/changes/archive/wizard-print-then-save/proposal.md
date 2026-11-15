---
cairn: change
id: wizard-print-then-save
status: landed
created: 2026-08-11
---

# Print the account, then offer to save it

## Why

The wizard's last prompt reads `Save this configuration to a file (no prints it)?`. The parenthetical exists because the prompt decides two things at once: whether the account is written to a file, and whether it is printed at all. So the user has to be told what the no-arm does, and the account stays invisible until after the decision is made. Choosing where a configuration goes before having seen it is the wrong order.

## What

Print the fragment first, always, then offer to save it. The prompt loses its parenthetical and its second meaning: `Save this configuration to a file?`, where no simply means the wizard stops and leaves the printed fragment for the user to place.

Stdout carries the fragment in every mode, which is what a redirect and `--json` already relied on; the terminal case now matches them instead of making printing conditional.

Confirm the append too, when the chosen file already holds something. It is the user's config, with their accounts and comments in it, and the wizard writing into it unasked is the one irreversible thing the flow does. Declining ends the wizard: the fragment is printed, so nothing is lost.

## Non-goals

- Prompting under `--json` or a redirected stdout. Those stay non-interactive.
