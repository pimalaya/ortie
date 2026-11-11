---
cairn: delta
change: the-shared-kit
---

## ADDED Requirements

### Requirement: Tool annotations
Each MCP tool SHALL carry `readOnlyHint`, `destructiveHint` and `openWorldHint`. `openWorldHint` SHALL be true on the tools reaching the authorization server (`auth_get`, `auth_resume`, `token_refresh`, `token_revoke`) and false on `token_inspect`, which reads the account's storage only, and `auth_status`, which reports the server's own flow. `token_inspect` and `auth_status` alone SHALL be read-only, and `token_revoke` alone destructive.

## MODIFIED Requirements

### Requirement: Command grammar reuse
Each stdin line SHALL be split as a shell splits it, quotes and escapes included, a word opening on `#` starting a comment to the end of the line, then parsed with the same `token` and `auth` command grammar as the one-shot CLI, subcommand prefixes inferred as there. A line left without words SHALL be skipped. A line setting a flag the session fixed when it started, `-a`, `-c` or a log flag, SHALL be refused naming that flag. A line that is not UTF-8 or whose quote is not closed, a parse error or a command error SHALL be reported and the loop SHALL continue, never terminating the process.

#### Scenario: A session flag on a line
- GIVEN a running `ortie repl` session
- WHEN `token show -a other` is entered
- THEN it fails with "Cannot set `-a` inside a session, start a new session with it instead", and the next line runs on the session's account

### Requirement: Interactive versus piped output
Every result and every failure SHALL be printed through the printer on stdout, flushed, the failure as the failure object (`ortie-error` under `--json`), whether stdin is a TTY or piped, and the loop SHALL continue; under `--json` each SHALL be one JSON value per line. When stdin is a TTY the session SHALL render the prompt `ortie> ` on stderr, and a line clap cannot parse SHALL get clap's message on stderr; piped, that parse error SHALL be printed on stdout as any failure. Help and version SHALL be no failure: printed on stdout in a text session, on stderr in a `--json` one, so its stdout stays one JSON value per line.

### Requirement: An MCP server over stdio
`ortie mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC message per line, with stdout carrying protocol messages only. `initialize` SHALL answer the client's protocol version when the server speaks it, 2025-06-18 otherwise, and SHALL carry instructions naming the order of the tools. A message that is not JSON SHALL answer -32700; one that is not a JSON object, a batch included, or whose `jsonrpc` is not `"2.0"` -32600; an unknown method -32601. The account SHALL be resolved at start, and its stored token read at most once for the server's lifetime, as in the REPL. A failure resolving the account, or one ending the server, SHALL be printed on stderr with exit status 1, never on stdout; a person missing a configuration meets the welcome instead.

### Requirement: A tool's output is the command's payload
A successful call SHALL return its JSON value as `structuredContent` and as text, described as `outputSchema` by the json-schema registry entry of the command it runs, or by the schema of the `auth_get` and `auth_status` answers, each derived as printed (the serialize contract): a key the answer always prints is required, `null` or not. A failed call SHALL return `isError: true` with the `ortie-error` object. An argument the tool does not declare SHALL be refused naming the declared ones, and a missing or non-string one naming it; an unknown or unlisted tool SHALL be a JSON-RPC error (-32602).

### Requirement: Account resolution failures name what is missing
Each of the four ways account resolution fails SHALL name what is missing or wrong and what to do about it: a missing configuration names the path it looked for and points at config.sample.toml, a missing named account lists the accounts the configuration does hold, sorted, a missing default names both ways of picking one, and several accounts marked `default = true`, none named, are refused naming them, sorted, rather than one taken.

#### Scenario: Two default accounts
- GIVEN a configuration whose accounts `a` and `b` both set `default = true`
- WHEN `ortie token show` runs without `-a`
- THEN it fails with "Several accounts are marked default (a, b), keep `default = true` on one, or name one with `-a <NAME>`", and `ortie -a b token show` runs on `b`

### Requirement: Text output ends with a newline
Every command's text rendering SHALL end with a newline, `token show` included, so the shell prompt starts on a line of its own. The REPL SHALL add none of its own: interactive, its prompt `ortie> ` follows the output on a line of its own; piped, results follow each other line after line.

## REMOVED Requirements
