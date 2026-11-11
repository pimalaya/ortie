---
cairn: log
change: repl-and-mcp-polish
landed: 2026-11-11
---

# REPL and MCP polish

`ortie repl` takes pimalaya-cli's repl-and-mcp-polish through the ../cli patch, with no source change: a command's help names the binary (`help token show` renders `Usage: ortie token show [OPTIONS]`, `token --help` renders `Usage: ortie token [OPTIONS] <COMMAND>`, both `Usage: token ...` before), and a text error hints `Restart the session with --log-level to enable more verbose logs` instead of `Run with --log-level ...`, a line setting `--log-level` being refused.

tests/repl.rs `the_help_goes_to_stderr_in_a_json_session` asserts both usage lines in full; its old `Usage: token` check fails against the new output, the new one against the old. src/cli.rs keeps its MCP startup match (pimalaya-cli adds no helper for the missing-configuration case), and src/mcp.rs builds no content item by hand, so the new `mcp::text` is not used.

Capabilities moved: repl (cairn/spec/repl.md: "Interactive versus piped output").

Verified: `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, `cargo test --all-features` (the live suites stay ignored).
