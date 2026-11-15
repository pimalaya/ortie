---
cairn: log
change: repl-takes-the-whole-grammar
landed: 2026-11-15
---

# The REPL takes the binary's whole grammar

`ortie repl` parses each line with `cli::Command`, the one-shot tree, run over a `Session` (src/cli.rs) holding the configuration paths, the `-a` name and the account, taken up front in a session and on first use on the command line; `Command::execute` serves both. The reduced `ReplCommandTree` and `ReplAuthCommand` are gone. pimalaya-cli 0.5.2 refuses a `repl` or `mcp` line, coded `usage-invalid`, and answers `-V`/`--version` from the version now on the command enum. The `repl` help takes the family wording.

Evidence: tests/repl.rs `a_line_takes_the_whole_grammar` (`account list`, `account check`, `completion bash`), `a_nested_session_is_refused_coded` and `a_version_line_prints_the_version` each fail on the reduced tree, which refused those lines as unknown subcommands or arguments. The existing REPL tests pass unchanged, the store still read once per session.

Capabilities moved: repl ("A session never nests" added; "Command grammar reuse", "Interactive versus piped output"), output ("Commands carrying a schema").
