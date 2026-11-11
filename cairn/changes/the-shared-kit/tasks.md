---
cairn: tasks
change: the-shared-kit
---

- [x] Cargo.toml: `[patch.crates-io]` on ../cli and ../config, pimalaya-cli `repl` and `mcp`, pimalaya-config `sample` for the tests, shlex dropped
- [x] `clap::commands` imports on the module paths
- [x] Welcome, `shipped_sample`, `is_wanted`, `shell::{quote, invocation}`; welcome output byte-identical to HEAD's
- [x] `ConfigFlags` for `-c`
- [x] `TomlConfig::load` and `take_account`, `ConfigMissingError` and `AccountError` from pimalaya-config; bare invocation unchanged; several defaults refused, tested
- [x] Sample test on `pimalaya_config::sample::variants`, variant count reported
- [x] REPL on `pimalaya_cli::repl::Repl`; tests for the refused session flag, the comment line, help on stderr in a JSON session
- [x] MCP on `pimalaya_cli::mcp`, `CapturePrinter`, `ToolResult`, `tool_output_schema`, serialize-contract schemas, `exit_on_failure`, `openWorldHint` per tool
- [x] Dead code deleted
- [x] fmt, clippy `-D warnings` (all features, and `--no-default-features --features rustls-ring`), `cargo test --all-features`, `cargo deny check`
- [x] `json-schema` keys unchanged against HEAD, every file passing `check-jsonschema --check-metaschema`
- [x] Specs, CHANGELOG, main.rs header; delta folded, log written
