---
cairn: log
change: repl-shell-split
date: 2026-11-04
---

# REPL lines split as a shell splits them

`ReplCommand::execute` (src/repl.rs) splits each line with `shlex::split` instead of `split_whitespace`, as himalaya's REPL does; an unclosed quote is a reported error and the session goes on.

Capabilities moved: `repl` (Command grammar reuse modified).

Verified: `lines_are_split_as_a_shell_splits_them` in tests/repl.rs fails against the whitespace split (`token "show"` was refused) and passes now.
