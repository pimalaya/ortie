---
cairn: change
id: the-google-application-ships
status: landed
created: 2026-11-28
---

# The Google application ships

## Why

Pimalaya's Google Desktop client `803561575535-41415ghqi04uibrl6t5umg3p27stdhqr.apps.googleusercontent.com` is in a Google project In production, pending Google's OAuth verification, registered for `https://mail.google.com/`, `https://www.googleapis.com/auth/calendar`, `https://www.googleapis.com/auth/contacts` and `https://www.googleapis.com/auth/tasks`. The README still leaves the Google client empty and says the application "is not released yet", that refresh tokens expire after 7 days, and lists `calendar.events`, `calendar.calendarlist.readonly` and `carddav` among the scopes.

Shipping it has a condition. Google's Workspace API user data policy ("Transparent and accurate notice and control", read on 2026-11-06, android cairn/changes/oauth-verification) requires a disclosure of the data accessed and how it is used "within the application itself", in the normal flow and not behind a menu, that "cannot be placed only in a privacy policy or terms of service", and that "must accompany and immediately precede a request for user consent", requiring "affirmative user action (for example, tap to accept, tick a check-box...)". The Android app shows "Pimalaya and your Google data" (Agree / Not now, a privacy policy link) before the browser opens. Ortie prints an authorization URL, or a user code, and nothing before it; pimalaya-applications-lead left this disclosure "owed before the next release".

## What

### The disclosure

- **Trigger.** An account whose `client-id` is Pimalaya's Google Desktop client, on a grant asking the user's consent: `authorization-code` or `device`. The client credentials grants ask no one, and an application of the user's own is the user's, so neither shows it. `auth listen` and `auth resume` finish a flow `auth get` started, after its disclosure, so they show none.
- **Where.** `auth get` (the command line and a REPL line) before anything else: before binding a listener, building the authorization URL or requesting a user code. The MCP `auth_get` tool before it replaces the pending flow.
- **Text.** Titled "Pimalaya and your Google data", as on Android. One row per Google service the account's scopes reach, all four when none is recognised: Gmail (your messages, labels and sending addresses), Google Calendar (your calendars and events), Google Contacts (your contacts and labels), Google Tasks (your task lists and tasks). Then the use: only to carry out your own actions on this device; Pimalaya runs no server, the data goes between Google and this device only, never sold or used for ads. Then the privacy policy, https://pimalaya.org/privacy/, whose "Google user data" section it summarises.
- **Interactive.** When stdin and stdout are terminals, `--json` is off and no session reads stdin, `auth get` prints the disclosure on stdout, the human path, then `Agree and continue? [y/N] `. `y` or `yes`, any case, goes on; anything else, an empty line and EOF included, fails coded `agreement-missing`, exit 1, nothing sent.
- **Affirmative flag.** `auth get --agree-data-use` is the user agreeing beforehand: no prompt, and in text the disclosure is still printed before the authorization URL or the user code, so it accompanies the consent request. Under `--json` it prints nothing more than today.
- **Never skipped silently.** Without the flag, where no prompt can be asked (`--json`, stdin or stdout not a terminal, a REPL session whose stdin carries its lines), `auth get` fails coded `agreement-missing`, exit 1, before anything is sent, its message being the disclosure in one sentence, the privacy policy's link and the way to agree, so a script or a frontend shows the user what they agree to before passing the flag.
- **MCP.** `ortie mcp --agree-data-use`, which the user writes in the agent's server configuration, is the affirmative action: an agent cannot set it through a tool call. Without it, `auth_get` on such an account answers the same `agreement-missing` failure, naming the server flag, and the pending flow stays. With it, the `auth_get` answer carries `disclosure`, the one-sentence text, which the instructions tell the agent to show the user with the authorization URI or the user code; `null` on any other account.
- **Code.** `agreement-missing` joins the vocabulary: exit 1 like any failure object, exit 2 staying for a command line the grammar refuses.

### The shipped client

- README's Google section gives Pimalaya's client id and its client secret (a Desktop client secret is not confidential, Google's installed-application documentation and RFC 8252 section 8.5; it was shipped in the 3.0.0 development cycle and taken from ortie's history), with exactly `https://mail.google.com/`, `https://www.googleapis.com/auth/calendar`, `https://www.googleapis.com/auth/contacts` and `https://www.googleapis.com/auth/tasks`. Its text says the application is in production pending Google's verification: an unverified-app warning at consent and a 100-user cap until then, and the disclosure `auth get` shows first.
- config.sample.toml keeps its one generic `example` account (cairn/spec/config.md), whose client keys already point at the README for Google; there is no config.samples/ directory any more.
- cairn/spec/provider-quirks.md: the application is registered for the full `calendar`, so CalDAV works with it; still not CardDAV, whose `carddav` scope it is not registered for.

## Not in this change

- The verification itself (demo video, console texts): android cairn/changes/oauth-verification.
- A disclosure for other providers: none asks for one.
- MCP elicitation (a server-initiated confirm in the client): pimalaya-cli's MCP module answers requests only.
