---
cairn: log
change: the-shared-kit
landed: 2026-11-11
---

# The shared kit

Ortie runs on the shared pimalaya-cli and pimalaya-config code, Cargo.toml patching both to ../cli and ../config (pimalaya-cli gains the `repl` and `mcp` features, pimalaya-config the `sample` feature for the tests; shlex leaves the direct dependencies, unicode-width arrives through pimalaya-cli). In src, tests and Cargo.toml, 781 lines left and 330 came.

- src/cli.rs builds the shared `welcome::Welcome` with Ortie's intro, next step and `auth get`; on eight inputs (download and shipped sample, default path, `-c` with a quote and a space, two `-c`, `-a`) its output under a pseudo-terminal is byte-identical to the HEAD binary's. `-c` is `ConfigFlags`. Account resolution is `TomlConfig::load` then `take_account`, the shared `ConfigMissingError` downcast by main.rs and the bare invocation still showing the help over a broken file. `shipped_sample`, `find_sample`, `ConfigPathsArg`, Ortie's `ConfigMissingError` and account lookup, and their tests left.
- src/auth/get.rs prints its handoff commands with `shell::quote`, src/auth.rs and the REPL start them with `shell::invocation`; `shell_quote`, `invocation` and their tests left.
- src/config.rs implements `sample_url` and `accounts` in place of `take_default_account`. Several `default = true` accounts are refused naming them: the HEAD binary took `a` five times and `b` once in six runs on such a file. The sample test runs `pimalaya_config::sample::variants`: 60 variants, the same names and texts as Ortie's own helper produced.
- src/repl.rs is `Repl::new("ortie", json).run` over the `token` and `auth` tree. Command errors print on stdout interactive too, `#` comments are skipped, prefixes are inferred, `-a`, `-c` and the log flags are refused by name, help goes to stderr in a `--json` session, and the prompt is `ortie> `. Three new tests (session flag, comment and prefix, help) fail against HEAD and pass.
- src/mcp.rs answers the protocol with `mcp::answer` and `send`, reads stdin with `spawn_stdin_reader` into its own channel beside the listener and device poll events, runs commands into `CapturePrinter` (`ToolResult::printed`, `into_single` naming the right count), and runs under `exit_on_failure`: a missing configuration, two defaults or a broken file now print on stderr with an empty stdout, the welcome kept for a person. `auth_get` and `auth_status` take the serialize contract `output_schema`, which adds their always-printed nullable keys to `required`; the rest of `tools/list` is unchanged. The `McpRunError::Report` arm in `exchange` was dead (`run` only built `Error`) and left with the type. `openWorldHint` is declared per tool with unchanged values: true for the four tools reaching the authorization server, false for `token_inspect` (storage only) and `auth_status`.

`json-schema -d` writes the same eleven files, byte-identical to HEAD's, each passing `check-jsonschema --check-metaschema`.

Capabilities moved: repl (cairn/spec/repl.md: "Command grammar reuse", "Interactive versus piped output"), mcp (cairn/spec/mcp.md: "An MCP server over stdio", "A tool's output is the command's payload", "Tool annotations" added), config (cairn/spec/config.md: "Account resolution failures name what is missing"), output (cairn/spec/output.md: "Text output ends with a newline").

Verified: `cargo fmt`, `cargo clippy --all-targets -D warnings` with `--all-features` and with `--no-default-features --features rustls-ring`, `cargo test --all-features`, `cargo deny check` (the getrandom duplicate and the unmatched `allow-org.github` warnings, as at HEAD).
