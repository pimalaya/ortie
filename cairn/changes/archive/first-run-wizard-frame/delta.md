---
cairn: change
change: first-run-wizard-frame
---

# Delta

## ADDED Requirements

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: The offer is a hook, not a gate
A missing configuration SHALL raise an offer to generate one, from a bare invocation and from any command needing an account. The offer never ends the process: a command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail on the configuration it still has not got. A bare invocation has nothing to carry on to, so a declined offer falls back to the help. Nothing is offered when stdin is not a terminal or `--json` is set.

### Requirement: A bare invocation meets the newcomer, not the user
A bare `ortie` SHALL raise the offer when it finds no configuration, and show the help when it finds one, since someone already set up is asking what the commands are. `--account` with no subcommand is a half-typed command and shows the help too.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `ORTIE_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run. It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: A generated account takes a free name and one default
The account name SHALL be suffixed until the configuration does not already hold it, since a second `[accounts.<name>]` table makes the whole document fail to parse. The generated account claims `default` only when no other account does, since two defaults resolve to whichever the account map yields first.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: The configuration path is read from the environment
The configuration path SHALL be read from `ORTIE_CONFIG` as well as `-c`, both accepting a `:`-delimited list merged in order.

## MODIFIED Requirements

### Requirement: The wizard saves where the configuration lives
The save SHALL NOT prompt for a path: it writes where `-c` or `ORTIE_CONFIG` pointed, or the default location. A file already holding accounts is still appended to rather than overwritten, and still confirmed before it happens, since it is one the user already owns. The fragment still reaches stdout before the save is offered, so the choice is made having seen what is being placed.

## REMOVED Requirements
