---
cairn: change
id: repl-and-mcp-polish
status: landed
created: 2026-11-11
---

# REPL and MCP polish

## Why

pimalaya-cli's repl-and-mcp-polish change fixes two things an `ortie repl` session showed: a command's help read `Usage: token show [OPTIONS]` without the binary, and a text error hinted `Run with --log-level`, a flag a line is refused.

## What

- Ortie takes both through its ../cli patch, no source change: the help reads `Usage: ortie token show [OPTIONS]`, a text error hints `Restart the session with --log-level to enable more verbose logs`.
- tests/repl.rs asserts the full usage lines.
- The MCP startup in src/cli.rs keeps its match: pimalaya-cli adds no helper for the missing-configuration case. `mcp::text`, now public, is not needed: Ortie builds no content item by hand.
