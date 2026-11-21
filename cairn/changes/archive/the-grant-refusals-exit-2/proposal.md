---
cairn: change
id: the-grant-refusals-exit-2
status: landed
created: 2026-11-21
---

# The grant refusals exit 2

## Why

The contract check of 2026-11-21, before the 3.0.0 release, ran every command and subcommand of Ortie under `--json` against the kit's output and error contract (pimalaya-cli cairn/spec/output.md, error.md, repl.md, mcp.md). Every shape conformed. One class of exit codes did not: the 2026-11-20 rulings (ORT-X2) moved two refusals of the command line made after clap parsed it to `usage-invalid` and exit 2 (`--handoff` on a grant without one, `auth listen` with no redirection), but their siblings stayed exit 1 uncoded. `auth resume --state` on a device account exited 1 while `auth resume --handoff` on the same account exited 2, for the same reason. pimalaya-cli error.md rules that a command line a binary refuses after parsing exits as one clap refused; changing the status after 3.0.0 would be a break, so it lands now.

## What

- `auth listen` on a device or a client credentials account, and on a redirection that is not an http URI on a loopback host (from `--redirect-uri` or `endpoints.redirection`, as the missing redirection already was), fail coded `usage-invalid`, exit 2.
- `auth resume` on a client credentials account, `--state`, `--pkce` or `--redirect-uri` on a device account, an empty device code and a redirected URI that does not parse fail coded `usage-invalid`, exit 2.
- The MCP tool `auth_resume` given a URI that does not parse answers its failure object coded `usage-invalid`, the flow still pending.
- One helper, `error::usage`, builds those refusals (and the two of ORT-X2, and the MCP argument refusals), the wording unchanged.
- Kept: the messages, a refusal never echoing a redirected URI (it carries the code).
