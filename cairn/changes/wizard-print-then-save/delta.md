---
cairn: delta
id: wizard-print-then-save
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Prints, then offers to save
The wizard SHALL emit a bare, valid `[accounts.<name>]` TOML fragment on stdout in every mode, with no leading comments: the guidance that used to head it lives in the stderr welcome banner, and every prompt and spinner renders on stderr. Under `--json` a JSON object is emitted instead so scripts can consume the discovery. When writing to a terminal the wizard SHALL then offer to save the account it just printed, through a prompt naming a single action and defaulting to saving, seeded with `$XDG_CONFIG_HOME/ortie/config.toml`. The prompt SHALL NOT decide whether the account is printed: it has already been, so declining leaves the user with the fragment to place themselves. A redirected stdout and `--json` SHALL stay non-interactive, with no save prompt, so `ortie >> <config>` keeps working.

### Requirement: Saves by appending, never overwriting
When the user accepts the save, the account SHALL be appended to the chosen file, separated from what precedes it, and a missing file (with its parent directory) SHALL be created. An existing file SHALL NOT be overwritten: the fragment is one `[accounts.<name>]` table, so appending adds an account and leaves the accounts and comments already configured untouched, which is what `ortie >> <config>` does by hand. A file that already holds something SHALL have the append confirmed before it happens, naming the path, since it is a file the user already owns; declining SHALL end the wizard without writing, the account having been printed already. An empty or missing file SHALL NOT be confirmed, having nothing to preserve.

## REMOVED Requirements

None.
