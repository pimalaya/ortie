---
cairn: log
change: a-failing-storage-command-hides-its-stdout
landed: 2026-11-19
---

# A failing storage command never reports its stdout

Finding ORT-1 of the CLI family analysis of 2026-11-15, under the owner's COH-1 ruling of 2026-11-18. src/account.rs reports a failing `storage.read.command` or `storage.write.command` by its trimmed stderr, or "<program> failed with exit status: <n>" when it wrote none, and never by its stdout, where the token is. The read side ends with ", run `<command>` yourself to see why", as pimalaya-config's `Secret::get` does.

Evidence: tests/account.rs `a_failing_read_command_never_reports_its_stdout` (`token inspect --json` and `account check --json`, with and without a stderr), tests/mcp.rs `token_inspect_never_reports_a_failing_read_command_stdout`, tests/client_credentials.rs `a_failing_write_command_never_reports_its_stdout` (`auth get` with a write command `cat; exit 1`). All three failed on 58de3a9, which printed the token.

Capabilities moved: config ("Storage commands").
