---
cairn: delta
id: wizard-append-and-argv
---

# Delta

## ADDED Requirements

### Requirement: Commands emitted as argv where they can be
A command the wizard builds from a known credential provider SHALL be emitted as an exec-style array, so no shell reinterprets an entry name. A shell string SHALL be used only where the command genuinely needs one: the write half of a provider pair that relies on shell features (`$(cat)` on macOS), and any command the user typed by hand. Emitted TOML values SHALL be escaped for the shape they are written in, so a value carrying a quote or a backslash still parses back.

### Requirement: Saves by appending, never overwriting
When the user accepts the save, the account SHALL be appended to the chosen file, separated from what precedes it, and a missing file (with its parent directory) SHALL be created. An existing file SHALL NOT be overwritten and no overwrite confirmation SHALL be asked: the fragment is one `[accounts.<name>]` table, so appending adds an account and leaves the accounts and comments already configured untouched, which is what `ortie >> <config>` does by hand.

## MODIFIED Requirements

### Requirement: Prints, or appends to a file
The wizard SHALL emit a bare, valid `[accounts.<name>]` TOML fragment with no leading comments: the guidance that used to head it lives in the stderr welcome banner, and every prompt and spinner renders on stderr. When writing to a terminal it SHALL offer to save the account to a file, through a prompt naming both arms (the yes-arm action, and that no prints instead) and defaulting to saving, seeded with `$XDG_CONFIG_HOME/ortie/config.toml`. A redirected stdout and `--json` SHALL stay non-interactive and print, with no save prompt, so `ortie >> <config>` keeps working; under `--json` a JSON object is emitted instead so scripts can consume the discovery.

### Requirement: Welcome banner frames the wizard
Bare `ortie` SHALL open with a welcome banner on stderr, before the first prompt, stating what Ortie is, what the wizard does, that it can save the account or print it, where every configuration field is documented, and that `ortie auth get` is what authorizes the account afterwards. The banner renders on stderr so it never pollutes a redirected fragment, and it is skipped under `--json`.

## REMOVED Requirements

### Requirement: Prints, never writes
**Reason**: replaced by "Prints, or appends to a file". The wizard now offers to save, which the old requirement forbade outright. What that requirement was protecting, a config the user owns, is preserved by appending rather than rewriting, and by keeping the redirected and JSON paths print-only.
