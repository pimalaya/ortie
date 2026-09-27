---
cairn: delta
change: wizard-append-or-print
---

# Delta

## ADDED Requirements

### Requirement: Saves, appends, or prints
The wizard SHALL emit a bare, valid `[accounts.<name>]` TOML fragment with no leading comments: the guidance that used to head it lives in the stderr welcome banner, and every prompt and spinner renders on stderr. Under `--json` a JSON object carrying the same account is emitted instead, so scripts can consume the discovery.

`--json` and a redirected stdout SHALL print the account and touch no file, which is what keeps `ortie configure > <config>` working. Writing to a terminal, the account SHALL instead be handed to a single prompt naming one action and defaulting to it: appending, when a configuration file is already there, saving otherwise, where the config capability says the configuration lives. Declining SHALL print the account on stdout rather than write it, so nothing the wizard resolved is lost and the fragment is still there to place by hand.

## MODIFIED Requirements

### Requirement: A custom application is not prompted for
The custom entry SHALL prompt for nothing: not the client id, not its secret, not the redirection endpoint. Registering an application of one's own is the rare path, and whoever took it is already editing the configuration, so typing those fields into a wizard only to check them in a file afterwards helps nobody. The account SHALL be emitted with everything the wizard did resolve (grant, endpoints, discovered scopes, storage) and an empty `client-id`.

An account left without an application SHALL be explained on stderr immediately before the account is placed, so the offer is answered, and the fragment read, knowing what is missing from it. The explanation SHALL stay short: the fields to fill in (`client-id`, plus `client-secret.raw` and `endpoints.redirection` where the provider requires them) and the documented sample configuration, nothing more. It SHALL NOT be told to run `ortie auth get`, which cannot succeed until the client id is filled in.

### Requirement: Saves by appending, never overwriting
When the offer is accepted, a configuration file already there SHALL be appended to as plain text, separated from what precedes it by a newline that also terminates a last line ending without one, and a missing file (with its parent directory) SHALL be created.

An existing file SHALL NOT be overwritten, nor parsed and re-serialized: the account is one `[accounts.<name>]` table, so appending its text adds an account and leaves the accounts, comments, ordering and hand-written formatting already in the file exactly as they are, which is what `ortie configure >> <config>` does by hand.

A written account SHALL be confirmed on stderr, naming where it landed and under which name, pointing at `-a <name>` when another account holds the default, since the name was never asked for, and naming `ortie auth get` when the account has an application to authorize with.

### Requirement: The wizard saves where the configuration lives
The save SHALL NOT prompt for a path: it writes where `-c` or `ORTIE_CONFIG` pointed, or the default location. A file already holding accounts is appended to rather than overwritten, and confirmed before it happens in the one prompt the save step asks, since it is a file the user already owns. Declining SHALL print the account on stdout instead of writing it.

## REMOVED Requirements

### Requirement: Prints, then offers to save
Replaced by "Saves, appends, or prints". The account no longer reaches stdout in every mode: on the terminal path the prompt decides, and declining is what prints.
