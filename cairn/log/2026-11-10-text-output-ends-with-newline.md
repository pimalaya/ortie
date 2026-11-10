---
cairn: log
change: text-output-ends-with-newline
landed: 2026-11-10
---

# Text output ends with a newline

`TokenShowOutput` (src/token/show.rs) ends the token with `writeln!`, like every other output type, `TokenInspectOutput` included since the owner's 2c54319. `answer` (src/repl.rs) no longer prints a newline after a text result, only flushes, so an interactive session shows one blank line before each prompt, text and JSON alike, and a piped one prints results line after line. The REPL's `help` read developer notes ("One parsed REPL input line, the CLI grammar minus the binary name", "The `auth` leaves usable inside the REPL"); `ReplLine` and `ReplAuthCommand` carry user-facing help now.

Checked the same day, with no change needed: every `json-schema` schema passes the 2020-12 meta-schema (check-jsonschema 0.37.2); every command run with `--json` against a mock OAuth server (token show, inspect, refresh, revoke; auth get for the authorization code, device and client credentials grants; auth resume for code and device; auth listen; completion, manual, json-schema; four failures) prints one value valid against its schema, with stderr empty and failures exiting 1; a piped `--json` REPL session prints each line valid against its command's schema, its failure as `ortie-error`; `token inspect`, `show` and `refresh` against the owner's Google, Microsoft Graph, Microsoft IMAP and SMTP, and Fastmail test accounts validate too. An `ortie mcp --allow-revoke` session driven over stdio lists six tools whose input and output schemas pass the meta-schema, and each call's `structuredContent` matches its `outputSchema`, its text content the same value, through a code flow caught by the listener, one resumed from its URI, `auth_status`, `token_refresh` and `token_revoke`.

Found on the way, left to pimalaya-config: a `-c` path after the first one that does not exist is skipped with a debug log, so a mistyped private configuration goes unnoticed.

Capabilities moved: output (cairn/spec/output.md: "Text output ends with a newline" added), token (cairn/spec/token.md: "Show the raw access token" names the newline).
