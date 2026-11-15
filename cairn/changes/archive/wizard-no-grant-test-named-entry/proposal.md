---
cairn: change
id: wizard-no-grant-test-named-entry
status: landed
created: 2026-08-11
---

# Drop the wizard's grant test, name the storage entry

## Why

Two corrections to `wizard-himalaya-parity`.

**The grant test does not fit.** Himalaya tests its account by opening a connection with the credential it just collected: silent, instant, invisible. Transposing that to Ortie meant running the discovered grant, which is a different animal. It hijacks the browser mid-wizard, it cannot complete for a provider whose redirection the local listener cannot capture, it forces the wizard to duplicate a grant runner it cannot share with `auth get` (different output stream), and it writes a real token into the user's keyring before the user has even saved the config the token belongs to. The wizard's job is to hand back a config; `ortie auth get` is the command that authorizes it, and it already reports every failure the test would have caught. Skipping straight to it is one fewer thing to explain and one fewer thing to go wrong.

**The storage entry should be asked, not assumed.** The Himalaya wizard's credential picker is two steps: a strategy (which keyring), then the entry the secret lives under, seeded with a sensible default and used verbatim. Ortie's storage step only did the first, silently deriving `ortie/<account>` from the account name. That is a name the user never chose, cannot see before it appears in the fragment, and does not match an entry they may already hold for this account. Doing what Himalaya does removes the guesswork.

## What

- Remove `src/wizard/check.rs` and its call. The wizard runs no grant, prints no authorization line, and stops at the storage step.
- Add the entry prompt to the storage step, mirroring the shared picker's shape: `Token storage strategy:` (the credential provider CLIs available on the running OS, then custom shell commands), then `Token storage keyring entry:` defaulted to the account name.
- Use the entry verbatim, as Himalaya does, dropping the `ortie/` namespace the step used to prepend. A user wanting one types it.
- Point at `ortie auth get` in the welcome banner and the README, as the next step after saving the fragment.

## Non-goals

- Any lighter substitute for the test (an endpoint reachability probe, a storage round-trip). If the wizard is not going to prove the account works, it should not pretend to.
