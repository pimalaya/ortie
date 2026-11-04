---
cairn: log
change: repl-line-json
date: 2026-11-04
---

# A REPL line takes `--json`

`ReplLine` (src/repl.rs) flattens `JsonFlag`, and a line setting it in a text session runs and answers through a JSON `StdoutPrinter` of its own; the line answer moved to `answer`, shared by both printers.

Capabilities moved: `repl` (A line's JSON flag added).

Verified: `a_line_json_flag_applies_to_that_line` in tests/repl.rs fails against the previous grammar ("unexpected argument") and passes now. `a_piped_session_has_no_prompt_and_reads_storage_once` passes against both: the prompt renders only when stdin is a terminal, and three commands run the read storage command once.
