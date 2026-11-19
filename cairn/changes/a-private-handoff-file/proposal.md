---
cairn: change
id: a-private-handoff-file
status: active
created: 2026-11-19
---

# A private handoff file

## Why

Item 2 of the-handoff-leaves-the-command-line (finding ORT-4 of the CLI family analysis of 2026-11-15), parked by the owner on 2026-11-19. Since that change, `ortie --json auth get > flow.json` then `ortie auth listen --handoff flow.json` keeps the state and the PKCE verifier off the command line, but the file is created under the user's umask, readable by other local users on a 022 umask, and the human handoff still prints the commands with `--state` and `--pkce`.

## What (not implemented)

`auth get` writes the handoff itself to a file only its owner reads (a `--handoff <path>` flag, or a path of its own under the state directory), and the human handoff prints `ortie auth listen --handoff <path>` and `ortie auth resume --handoff <path> <REDIRECTED_URI>`. It waits for the kit's private-create helper (KIT-4: pimalaya-cli 0.6.1 has none), so the file is created 0600 the way every family app creates its private files.
