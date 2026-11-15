---
cairn: change
id: mcp-gates
status: landed
created: 2026-11-15
---

# MCP gates as the family's

## Why

pimalaya-cli's MCP spec (cairn/spec/mcp.md, "Gates list what a tool may do") has a server started without a gate list only the tools that read, `--allow-write` the ones changing the account's data, and leaves room for a narrower gate of a binary's own, naming Ortie's `--allow-revoke`. Ortie listed `auth_get`, `auth_resume` and `token_refresh`, all `readOnlyHint: false`, with no gate (finding F2, owner ruling D2), and set `openWorldHint` on every tool reaching the authorization server, where the family's one meaning is acting beyond the account's own data, reaching other people.

## What

1. **`--allow-write`** lists `auth_get`, `auth_resume` and `token_refresh`, which replace the stored token. Without a flag the server lists `token_inspect` and `auth_status` alone.
2. **`--allow-revoke`** stays the gate of `token_revoke` and now requires `--allow-write`, as calendula's `--allow-send` does: revoking is a narrower write. A `--allow-revoke` alone is refused by clap (exit 2) naming `--allow-write`, rather than starting a server that silently lost the writing tools.
3. **`openWorldHint` is false on every tool**: none acts beyond the account's own token. Reaching the authorization server is reaching the network, which the hint does not mean. The read-only and destructive hints keep their values and move with no gate.
4. **The instructions name the gates and the failure object**, as himalaya's, cardamum's and calendula's do: the writing tools under `--allow-write`, `token_revoke` under `--allow-revoke` as well, and a failed call answering `isError` with an `ortie-error` object.
5. **A refused argument in the kit's words.** Ortie writes its tool inputs by hand, no tool running a clap command, so pimalaya-cli's `Grammar` and `usage_error` do not apply. Its own refusals already read as the kit's (``Missing argument `uri` ``, ``Argument `uri` must be a string``); a `null` `uri` now reads as missing, as the kit reads a `null` required argument.

`ortie mcp` is new in v3 (2.3.0 has none), so this breaks no released user; the MIGRATION.md bullet of `ortie mcp` and its CHANGELOG line describe the gates.
