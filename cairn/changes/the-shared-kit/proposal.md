---
cairn: change
id: the-shared-kit
status: landed
created: 2026-11-11
---

# The shared kit

## Why

Ortie grew its own copies of what every Pimalaya CLI needs: the first-run welcome and the sample lookup, the shell quoting of a printed command, the `-c` flag, account resolution, the sample-variants test helper, a REPL loop and the JSON-RPC half of an MCP server. pimalaya-cli and pimalaya-config now carry each of them once for the family (the welcome checked byte-identical to Ortie's on eight inputs), so a fix lands in every CLI at once and Ortie keeps only what is Ortie's: its commands, its sentences and its MCP flow state machine.

Moving onto them also fixes what the copies got wrong or never did:

- Several accounts marked `default = true` were not refused: the default was whichever the hash map iterated first.
- An interactive REPL printed a command error on stderr as a Rust `Debug` dump, apart from the stdout every other answer uses.
- A REPL line setting `-a` or `-c` failed with clap's generic "unexpected argument", and a `#` line was parsed as a command.
- The MCP `auth_get` and `auth_status` output schemas came from `schemars::schema_for!`, the deserialize contract, so their always-printed `null` keys were not required, unlike every registry schema.
- A failure resolving the MCP server's account went to stdout, which carries protocol messages only.
- A command printing several values made a tool fail with "printed {n-1} values".

## What

- Cargo.toml patches pimalaya-cli and pimalaya-config to ../cli and ../config, enabling pimalaya-cli's `repl` and `mcp` features and pimalaya-config's `sample` feature for the tests.
- src/cli.rs builds a `pimalaya_cli::welcome::Welcome` with Ortie's sentences, uses `ConfigFlags` for `-c`, and resolves the account with `TomlConfig::load` and `take_account`, whose `ConfigMissingError` and `AccountError` come from pimalaya-config. `shipped_sample`, `find_sample`, `ConfigPathsArg`, Ortie's `ConfigMissingError` and its account lookup are deleted.
- src/auth/get.rs uses `pimalaya_cli::shell::{quote, invocation}`; its own copies are deleted.
- src/config.rs implements `sample_url` and `accounts`, drops `take_default_account`, and its sample test runs `pimalaya_config::sample::variants`.
- src/repl.rs runs `pimalaya_cli::repl::Repl` over Ortie's `token` and `auth` tree; the loop, `ReplLine` and `answer` are deleted.
- src/mcp.rs keeps its hand-written tools and the `McpFlow` state machine, and answers the protocol through `pimalaya_cli::mcp` (`answer`, `send`, `spawn_stdin_reader` feeding Ortie's own event channel, `ToolResult`, `tool_output_schema`), captures commands with `CapturePrinter`, describes `auth_get` and `auth_status` with the serialize contract `output_schema`, and runs under `exit_on_failure`.
- `openWorldHint` stays true on the tools reaching the authorization server and false on the others, now declared per tool rather than derived from `readOnlyHint`.
- Specs repl, mcp, config and output, CHANGELOG and the main.rs header follow.
