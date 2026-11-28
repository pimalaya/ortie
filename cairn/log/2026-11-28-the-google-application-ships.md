---
cairn: log
change: the-google-application-ships
landed: 2026-11-28
---

# The Google application ships

Pimalaya's Google Desktop client `803561575535-41415ghqi04uibrl6t5umg3p27stdhqr.apps.googleusercontent.com`, in a Google project In production pending Google's OAuth verification, is the README's Google account, and Ortie shows the in-app disclosure Google's Workspace API user data policy requires before the consent request.

## What landed

- src/auth/disclosure.rs: `AuthDisclosure::of` owes the disclosure on that client id and a consent grant (`authorization-code`, `device`), its rows the services the scopes reach (Gmail, Google Calendar, Google Contacts, Google Tasks; all four when none is recognised). `obtain` takes `--agree-data-use` (the disclosure still printed in text), asks `Agree and continue? [y/N]` when stdin and stdout are terminals, `--json` is off and no session reads stdin, and refuses any other run with `refusal`, coded `agreement-missing`, its message the one-sentence disclosure, the privacy policy's link and the flag. A session refuses because the REPL holds the stdin lock its lines come through.
- src/auth/get.rs: `AuthGetCommand` gains `agree_data_use`; the agreement is obtained first, before a listener is bound, the authorization URL built or a user code requested.
- src/mcp.rs: `ortie mcp --agree-data-use`; without it `auth_get` on such an account answers the `agreement-missing` failure before the pending flow is cancelled. `McpAuthGetOutput.disclosure` carries the sentence, `null` elsewhere, and the instructions tell the agent to show it.
- src/error.rs: `ErrorCode::AgreementMissing`, exit 1.
- tests/disclosure.rs: the `--json` refusal on both grants with a listener counting connections (none), the flag and the unchanged handoff, the piped text order (disclosure before the request), another client, a piped REPL line, and under util-linux `script` a terminal declining, a terminal agreeing (the device request then sent), and a REPL in a terminal refusing without asking. tests/mcp.rs: the server flag, and `disclosure` null on another client. Unit tests: the trigger, the services, the prompt answers, the refusal's text and code.
- README.md: the Google section gives the client id and its secret (taken from 25e15f0, the 3.0.0 cycle's `KNOWN_APPS`, written without being printed), the scopes `mail.google.com`, `calendar`, `contacts` and `tasks` (`calendar.events`, `calendar.calendarlist.readonly` and `carddav` gone), the unverified-app warning and the 100-user cap until verification, CardDAV on a client of one's own, and the disclosure.
- config.sample.toml unchanged: its one generic account already points at the README for Google; config.samples/ no longer exists.

## Spec

- cairn/spec/auth.md: ADDED "Pimalaya's Google data disclosure precedes the consent".
- cairn/spec/mcp.md: ADDED "The MCP agreement is the server's flag".
- cairn/spec/output.md: "A failure carries a stable code" gains `agreement-missing`.
- cairn/spec/config.md: the README's Google block holds Pimalaya's client id and secret and the four scopes.
- cairn/spec/provider-quirks.md: "Current Google endpoints" names the shipped client, its verification status, the full `calendar` (CalDAV works) and no CardDAV.

## Left

- The commit 2884248 of 2026-11-10 says the production client "having been removed" and moved the sample to a development client; this change ships the 41415 client and its secret as the owner asked, assuming it was restored with its secret. No sign-in was run.
