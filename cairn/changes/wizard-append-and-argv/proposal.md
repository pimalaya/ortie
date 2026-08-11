---
cairn: change
id: wizard-append-and-argv
status: landed
created: 2026-08-11
---

# Save by appending, and emit commands as argv

## Why

Three corrections, the last two of which Himalaya should adopt back once they are proven here.

**Commands should be lists.** The Himalaya wizard records a known credential provider as an argv (a TOML array) and keeps a shell string for the fallback, so nothing in an entry name is reinterpreted by a shell on the way to the program. Ortie joined the argv it was handed back into a shell string, throwing that away for no gain: an entry with a space, a `$` or a `;` in it would be re-split or expanded on every read. The config already accepts both shapes, so the array costs nothing.

**"Save this configuration to a file, or print it?" does not say which answer is which.** Himalaya's prompt reads as a choice between two things but is answered yes or no, and neither answer is attached to an arm. Naming the yes-arm action and stating what no does removes the guess.

**Overwriting is the wrong move for a fragment.** Himalaya asks to overwrite an existing config, which is a strange thing to offer: the wizard produces one `[accounts.<name>]` table, and the file it is being written to is a config the user built, with other accounts and comments in it. Overwriting destroys all of that to save one account. Appending adds the account and leaves everything else alone, which is exactly what the documented `ortie >> <config>` does by hand. There is nothing to confirm, so the overwrite prompt goes away entirely.

## What

- Split `StorageCommand` into `Argv(Vec<String>)` and `Shell(String)`. A keyring read is an argv, rendered as a TOML array; a keyring write stays a shell line (the macOS pair needs `$(cat)`), as do hand-typed commands.
- Render every emitted TOML value through a helper that escapes what the shape requires: basic strings escape `\` and `"`, and a shell line stays a literal (single-quoted) string unless it carries a `'`.
- Offer to save at the end of the wizard, when writing to a terminal: `Save this configuration to a file (no prints it)?`, defaulting to yes, then a path prompt seeded with `$XDG_CONFIG_HOME/ortie/config.toml`.
- Append to an existing file rather than asking to overwrite it, separated by a blank line so two tables never glue together. A missing file (and its parent directory) is created.
- Keep the redirected-stdout and `--json` paths non-interactive: they print, with no save prompt, so `ortie >> <config>` is unchanged.

## Non-goals

- Rewriting or merging into an existing account of the same name. The wizard appends; a duplicate table key is the user's to resolve, and TOML will tell them.
