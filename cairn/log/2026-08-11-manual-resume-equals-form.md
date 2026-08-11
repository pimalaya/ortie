---
cairn: log
id: manual-resume-equals-form
date: 2026-08-11
---

# Print the manual resume command in the equals form

The `auth resume` command printed by `auth get` attaches its values to their flags with `=` now: `--state='...' --pkce='...'` instead of the space form.

A PKCE verifier is 43 bytes drawn from the RFC 7636 unreserved set, four of which (`-`, `.`, `_`, `~`) are not alphanumeric, so about one run in thirty produced a value starting with `-` or `~`. A state is URL-safe base64, whose alphabet also holds `-`, so one run in sixty-four started with it too. The single quotes already there stopped the shell expanding a leading `~`, but they never helped with the leading `-`: that is clap reading the value as a flag, which quoting cannot reach. Everything after `=` is the value, so the parser never gets the chance.

This is the printing half of the problem. The values themselves would be copy-paste-safe everywhere, `--json` output included (where nothing can be quoted for the user), if the generated alphabets dropped `-`, `.`, `_` and `~`. That belongs in io-oauth, which is consumed here as a released crate, so it waits for a release; RFC 7636 permits any subset of unreserved, and 43 alphanumeric characters still carry 256 bits.

Capabilities moved: auth.
