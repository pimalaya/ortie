---
cairn: change
id: the-flow-hardened
status: landed
created: 2026-11-19
---

# The authorization flows hardened

## Why

Four findings of the CLI family analysis of 2026-11-15 on the grant flows, each re-checked open on 2026-11-16 and on 2026-11-19 at 58de3a9:

- ORT-4, in part: an interactive `auth get` resolved an ephemeral redirect port by binding it, dropped the listener (src/account.rs) and bound again to wait, leaving the port free in between; and it printed the state and the PKCE verifier although it finishes the flow in process, where the device twin hides its code.
- ORT-5: an `https` loopback redirect counted as one the listener captures, though it speaks plain HTTP: `auth get` waited for ever, `auth listen` too, and MCP `auth_get` answered `redirectCaptured: true`.
- ORT-6: MCP `auth_resume` consumed the pending flow before reading the URI, so one mis-pasted URL cost a new consent.
- ORT-7: clap's refusal of a bad `--pkce` echoed the value, against the auth capability; the device handoff printed `auth resume <code>`, not runnable when the code starts with `-`.

## What

- The listener bound for an ephemeral port is the one that waits; the interactive text prints neither the state nor the verifier.
- Only an `http` loopback redirect is captured; an `https` one is finished through `auth resume`, `auth listen` refusing it.
- `auth_resume` refuses a URI that does not parse or does not echo the flow's state before taking the flow, which stays pending.
- `--pkce` is parsed by a clap value parser whose refusal names the flag and the length or byte, never the value; the device handoff prints `auth resume -- <code>`.

What ORT-4 leaves, the handoff leaving the command line, changes the flags of `auth listen` and is the proposal the-handoff-leaves-the-command-line.

### Breaking

None. `auth listen` on an https loopback redirect now fails at once where it waited for ever; the other changes are text and an MCP failure that keeps the flow.
