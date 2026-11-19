---
cairn: change
id: the-handoff-leaves-the-command-line
status: landed
created: 2026-11-19
---

# The handoff leaves the command line

## Why

Finding ORT-4 of the CLI family analysis of 2026-11-15 (LOW to MEDIUM, security; the handoff run, the exploit plausible). A non-interactive or `--json` `auth get` prints `ortie auth listen --state=... --pkce=<verifier> --redirect-uri=...`, and `auth listen` then waits, minutes at a time, with the state and the PKCE verifier in its argv. Another local user reading the process list (`ps`, /proc/<pid>/cmdline on a system without `hidepid`) can start an authorization of their own with that state and verifier, have the victim's listener catch their redirect, and have Ortie store their token under the victim's account. `auth resume <URI>` carries the authorization code on its command line too, for a shorter time.

The-flow-hardened (2026-11-19) closed the parts that change no flag: the interactive path keeps its listener and no longer prints the two values. What is left changes how `auth listen` and `auth resume` are given the flow, which no ruling covers.

## What

1. `auth listen` and `auth resume` read the flow from stdin when it is not a terminal and `--state`, `--pkce` are omitted: one JSON object, the `auth get --json` handoff as printed (`state`, `pkceCodeVerifier`, `redirectUri`), so `ortie --json auth get > flow.json` then `ortie auth listen < flow.json` works, and a script pipes one into the other. `auth resume` takes the redirected URI on stdin the same way when its positional is `-`.
2. The human handoff prints that form (`ortie auth listen < flow.json`), with `auth get --json` writing the file under a 0600 umask the user sets, or Ortie writing a 0600 file itself (`--handoff <path>`), which needs the kit's private-create helper (KIT-4, not in pimalaya-cli 0.6.1).
3. `--state` and `--pkce` stay, documented as visible to other local users, so nothing breaks; removing them would be breaking and is not recommended.

Ruled by the owner on 2026-11-19: 1 and 3 land, `auth resume` reading the flow object rather than the URI (the positional `-` was not taken up: the code is single-use and, under PKCE, useless without the verifier); 2 stays parked until the kit has its private-create helper, as the proposal a-private-handoff-file.

Ruled again by the owner the same day, once the implicit read had landed: no implicit read, since a script with an open stdin would hang. The handoff is read only when asked, `--handoff <PATH>`, a file or `-` for stdin as the family's JSON inputs take `-` (himalaya `jmap request`, cardamum's JSON inputs), and `--handoff` beside `--state` or `--pkce` is a clap conflict. A file path also serves the 0600 file of item 2 once the user creates it so; Ortie writing it stays parked.
