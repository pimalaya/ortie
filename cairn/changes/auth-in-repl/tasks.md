---
cairn: tasks
change: auth-in-repl
---

- [x] Convert `auth get`/`auth resume` (and their `execute_device`) to `&mut Account`
- [x] Resolve `let mut account` in `AuthCommand::execute` for the one-shot CLI
- [x] Add the `auth` branch (`get`/`resume`) to the REPL grammar, dispatched against the session account
- [x] Integration test: `auth get` → `auth resume` → `token show` in one REPL session
- [x] Update CHANGELOG; fold the delta into `cairn/spec/repl.md` and log
