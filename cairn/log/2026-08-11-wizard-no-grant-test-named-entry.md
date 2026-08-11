---
cairn: log
id: wizard-no-grant-test-named-entry
date: 2026-08-11
---

# Dropped the wizard's grant test, named the storage entry

Two corrections to `wizard-himalaya-parity`, landed the same day.

## The grant test is gone

`src/wizard/check.rs` is deleted, along with its module declaration and its call. The wizard now ends at the storage step and prints its fragment.

The transposition did not hold. Himalaya tests an account by opening a connection with the credential it just collected: silent, instant, invisible. Ortie's equivalent was running the discovered grant, which hijacks the browser mid-wizard, cannot complete when the redirection is not locally capturable, forces the wizard to duplicate a grant runner it cannot share with `auth get` (the wizard owns stdout for its fragment, `auth get` prints its progress there), and writes a real token into the keyring before the config that token belongs to has been saved anywhere. `ortie auth get` is the command that authorizes an account, and it already reports every failure the test would have caught. The welcome banner, the README and the crate header now point at it as the step after saving the fragment.

No lighter substitute was put in its place: a wizard that is not going to prove the account works should not pretend to.

## The storage entry is prompted

The storage step used to pick a credential provider and then silently derive `ortie/<account>` from the account name. It now follows the same two steps as the shared picker Himalaya's credential prompt goes through: `Token storage strategy:` (the provider CLIs available on the running OS, then custom shell commands), then `Token storage keyring entry:` seeded with the account name.

The entry is used verbatim, as in Himalaya, for both the read and the write command, so the `ortie/` namespace is no longer prepended: an entry the user already holds is named as it is, and anyone wanting a namespace types one. Custom shell commands are unchanged, still prompted as a read and write pair.

## Capabilities moved

- **discovery**: "Grant tested before printing" removed; "Storage entry named, not assumed" added; "Welcome banner frames the wizard" reworded to point at `auth get`; the overview no longer claims the wizard proves the account works.
