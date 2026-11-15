---
cairn: change
id: repl-takes-the-whole-grammar
status: landed
created: 2026-11-15
---

# The REPL takes the binary's whole grammar

## Why

pimalaya-cli's REPL spec (cairn/spec/repl.md) parses a line with the binary's command grammar, and himalaya, cardamum and calendula pass their whole command enum. Ortie parsed a reduced `token` and `auth` tree of its own (src/repl.rs), so a line of any other command was refused as an unknown subcommand (finding F3, owner ruling D3). pimalaya-cli 0.5.1 refuses a nested `repl` or `mcp` line itself, coded `usage-invalid`, and answers a `--version` line from the version the command enum carries, which leaves no reason for the reduced tree.

## What

1. **One command enum.** The REPL runs `cli::Command`, the one-shot tree, over a `Session` (src/cli.rs) holding the configuration paths, the `-a` name and the account once taken. The one-shot run and the session dispatch through the same `Command::execute`: an account-bound command takes the session's account, taken on first use on the command line and up front in a session, so the store is still read once per session. `account list` reads the configuration, `account check` checks the session's account, and the standard commands run as on the command line. The reduced `ReplCommandTree` and `ReplAuthCommand` go.
2. **Nested sessions.** A `repl` or `mcp` line is refused by pimalaya-cli, "Cannot start a session inside a session", coded `usage-invalid`, before Ortie runs anything; Ortie has no refusal arm of its own.
3. **The version.** `#[command(version, long_version = ...)]` on the command enum, so a `-V` or `--version` line prints what the command line prints.
4. **What a session still refuses.** A line setting `-a`, `-c` or a log flag, by name, as before.
5. **The help** of `repl` reads as the family's: "Start a session running commands over one account", ending on "`quit`, `exit`, or EOF ends it".
