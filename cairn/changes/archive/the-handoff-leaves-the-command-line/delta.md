---
cairn: delta
change: the-handoff-leaves-the-command-line
---

## ADDED Requirements
### Requirement: The handoff can stay off the command line
`auth listen` and `auth resume` SHALL take `--handoff <PATH>`, the object a `--json` `auth get` prints, read from that file or, for `-`, from stdin, as the family's other JSON inputs take `-`. It SHALL give the `state`, `pkceCodeVerifier` and `redirectUri` (the last unless `--redirect-uri` is given), its other keys ignored, so `ortie --json auth get > flow.json` then `ortie auth listen --handoff flow.json` keeps the flow's secrets off a command line other local users can read. `--handoff` beside `--state` or `--pkce` SHALL be refused as clap refuses a conflict (`usage-invalid`, exit 2). At most 64 KiB SHALL be read; an object that does not parse SHALL fail naming its line and column, never a value. `--handoff` SHALL be refused on any grant but the authorization code one, and `--handoff -` in a REPL session, which reads its lines from stdin. Without `--handoff`, neither command SHALL read stdin. `--state` and `--pkce` stay, their values visible to other local users while the command runs.
