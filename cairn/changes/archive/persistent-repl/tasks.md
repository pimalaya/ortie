---
cairn: tasks
change: persistent-repl
---

- [x] Add a `token: Option<...>` field to `Account` (built `None`); add a memoizing `resolve_token`
- [x] `write_to_storage`: stamp `issued_at = now()`, persist, then set the in-memory token; return the stamped token
- [x] `is_expired`: default a missing `expires_in` to 3600s
- [x] Refactor the `token` dispatch to `&mut Account` (`TokenCommand::execute`, `show`, `inspect`, `refresh::{execute,refresh}`); route reads through `resolve_token`
- [x] Add `src/repl.rs`: resolve-on-first-use, per-line parse of the `token` grammar (no exit on error), TTY prompt vs piped protocol, quit/EOF, zeroize on drop
- [x] Wire `Command::Repl` into `cli.rs` dispatch and `mod repl` in `main.rs`
- [x] Update README and CHANGELOG (no `config.sample.toml` change: `repl` adds no config fields)
- [x] Land: fold the delta into `cairn/spec/` (new `repl` capability, modified `token`) and append a log entry
