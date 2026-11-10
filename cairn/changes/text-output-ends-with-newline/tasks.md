---
cairn: tasks
change: text-output-ends-with-newline
---

- [x] `token show` ends with `writeln!` (src/token/show.rs); `token inspect` already did (the owner, 2c54319)
- [x] `answer` (src/repl.rs) only flushes a result
- [x] REPL help texts for `ReplLine` and `ReplAuthCommand`
- [x] Seen under `script` and piped: one blank line before each prompt, none between piped results
- [x] Token and output specs, CHANGELOG
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
