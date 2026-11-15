---
cairn: tasks
change: repl-takes-the-whole-grammar
---

# Tasks

- [x] src/cli.rs: `Session`, `Command::execute` over it, the version on the command enum; src/auth.rs takes the resolved account
- [x] src/repl.rs: the loop over `Command`, the account taken up front, the family help; the reduced tree removed
- [x] src/main.rs: the one-shot session
- [x] tests/repl.rs: `a_line_takes_the_whole_grammar`, `a_nested_session_is_refused_coded`, `a_version_line_prints_the_version` (each fails on the reduced tree)
- [x] Spec: repl, output; README, CHANGELOG, MIGRATION.md (Commands)
