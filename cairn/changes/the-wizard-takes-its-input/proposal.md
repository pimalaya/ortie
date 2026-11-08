---
cairn: change
id: the-wizard-takes-its-input
status: landed
created: 2026-11-08
---

# The wizard takes its input

## Why

The other Pimalaya CLIs (himalaya, cardamum, calendula, neverest) run no OAuth 2.0 grant themselves. Their wizards are about to hand OAuth off to Ortie: when the service the user picked needs OAuth and `ortie` is on `PATH`, they run `ortie configure <ADDRESS> --name <NAME> --service <SERVICE>...` then `ortie auth get -a <NAME>`, stdin and stderr inherited and Ortie's stdout redirected to their stderr, so Ortie still sees a terminal and saves as usual, and they write `["ortie", "token", "show", "-a", NAME]` as their token command.

`ortie configure` takes no argument today: it prompts for the address the caller already has, derives a name the caller cannot predict (suffixed when taken), and offers every grant discovery found, including ones for services the caller does not use. The caller's token command needs the name it chose, and its user should not be asked twice for an address nor arbitrate grants for services that tool does not speak.

## What

- `configure [INPUT]`: the email address, bare domain or issuer URL, given instead of prompted, then validated and discovered exactly as a typed answer is.
- `--service <SERVICE>`, repeatable: keeps only the grants tagged with at least one of the named services, by the wire names the pick list prints (`imap`, `pop3`, `smtp`, `jmap`, `caldav`, `carddav`, `webdav`, `managesieve`, `gmail`, `gcal`, `gpeople`, `msgraph`, `msgraphCalendar`, `msgraphContacts`); an unknown name is a usage error listing the valid ones. A grant resolved from a typed URL carries no service, the URL having named what it serves, and is kept. A filter leaving no grant stops the wizard as discovering nothing does, naming the input and the services. One grant left is carried through without a prompt.
- `--name <NAME>`: the account name, verbatim, never derived nor suffixed. When the configuration already holds it, the wizard asks (default yes) whether to keep that account, before anything else: kept, nothing changes and `{name, default, document: null}` is printed; declined, it fails naming the account. A free name proceeds as today, the default claim rule unchanged.
- Nothing else moves: no welcome for `configure`, the terminal check on stdin, the redirected-stdout and `--json` paths printing the account and touching no file.

## Compatibility

Additive on the command line. Under `--json`, `document` becomes nullable in the `ortie-configure` schema, `null` only on the kept path, which did not exist; the schema change rides the unreleased 3.0.0 that already reshaped this object.

## Left

The configuration checked for names, taken and held alike, is the file the wizard writes to, not every layer `-c a:b` or the default paths merge: a name held only in another layer is not seen. That predates this change (the suffix rule reads the same file).
