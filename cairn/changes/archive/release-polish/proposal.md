---
cairn: change
id: release-polish
status: landed
created: 2026-07-25
---

# Release polish

## Why
The refresh rotation window had never been audited, so a rotated refresh token could have been lost, and the README did not state the OAuth 2.1 posture Ortie follows. The change was written before the minor carrying the device grant; that minor shipped as 2.1.0 without it, so what is left lands before the next major instead.

## What
Audit of the write-after-refresh window in `token refresh`: the new token is written as soon as the server answers, with nothing fallible in between, and the previous refresh token is kept when no rotated one comes back. The window left is a failing storage write: a server rotating refresh tokens has invalidated the stored one by then. The token is now cached in memory before the write, so a REPL session keeps working on the issued tokens, and the error tells to run `auth get` if the next refresh fails. A README section states the posture: grants, PKCE default, exact redirect matching with the loopback variable-port exception, rotation.

Dropped as obsolete: the changelog notes for the PKCE default flip, the discover output shape and the lib removal (all in the 2.0.0 and 2.1.0 entries), and releasing the minor; the final OAuth 2.1 RFC is not published yet.
