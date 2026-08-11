---
cairn: log
id: wizard-append-and-argv
date: 2026-08-11
---

# Saved by appending, emitted commands as argv

Three corrections, landed the same day as the two changes they follow. The last two are meant to travel back to the Himalaya wizard once they have been used here.

## Commands are lists

`StorageCommand` is now `Argv(Vec<String>)` or `Shell(String)`. A keyring read is an argv, rendered as a TOML array:

```toml
storage.read.command = ["secret-tool", "lookup", "account", "posteo"]
```

Ortie used to join the argv the picker handed it back into a shell string, which threw away the one thing the argv buys: an entry name carrying a space, a `$` or a `;` is no longer re-split or expanded on every read. A shell string is kept only where the command genuinely needs one, the write half of a provider pair (`security add-generic-password -w "$(cat)"` bridges a secret the program takes as an argument) and anything typed by hand.

Emitted TOML values now go through escaping helpers rather than bare `format!` interpolation: basic strings escape `\` and `"`, and a shell line stays a literal single-quoted string, which is why the macOS write command reads as it does, falling back to a basic string when the line itself carries a `'`. A round-trip test writes a fragment and loads it back through the real config loader, so what the wizard prints is proven to be what the loader accepts, both command shapes included.

## The save prompt names its arms

The wizard now ends, when writing to a terminal, on:

```
Save this configuration to a file (no prints it)? (Y/n)
Configuration file path: /home/…/.config/ortie/config.toml
```

Himalaya's wording, "Save this configuration to a file, or print it?", reads as a choice between two things but is answered yes or no, with neither answer attached to an arm. Naming the yes-arm action and stating what no does removes the guess.

A redirected stdout and `--json` stay non-interactive and print with no prompt at all, so `ortie >> <config>` is unchanged.

## Saving appends, never overwrites

An existing file is appended to, separated by a blank line so two tables never glue together; a missing file and its parent directory are created. There is no overwrite prompt, because there is no overwrite.

Himalaya asks to overwrite, which is a strange thing to offer: the wizard produces one `[accounts.<name>]` table, and the file it is written to is a config the user built, with other accounts and comments in it. Overwriting destroys all of that to save one account. Appending adds the account and leaves the rest alone, which is exactly what the documented `ortie >> <config>` does by hand.

## Capabilities moved

- **discovery**: "Prints, never writes" removed, replaced by "Prints, or appends to a file"; "Saves by appending, never overwriting" and "Commands emitted as argv where they can be" added; "Welcome banner frames the wizard" reworded to mention the save.

## Follow-up

The save prompt wording and the append-instead-of-overwrite behaviour are to be ported back to the Himalaya wizard, which still asks the ambiguous question and still offers to clobber the file.
