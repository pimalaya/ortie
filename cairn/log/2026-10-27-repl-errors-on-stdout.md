---
cairn: log
change: repl-errors-on-stdout
landed: 2026-10-27
---

# A piped REPL reports errors on stdout

A piped `ortie repl` wrote command errors as `{err:?}` and parse errors as clap's message on stderr, against the family rule that data and errors share stdout. It now prints both through the printer as an `ErrorReport` on stdout, flushed, and continues: under `--json` one `ortie-error` object per line among the payload objects. Help and version requests keep clap's own stdout output, and the interactive TTY session keeps its prompt and errors on stderr.

tests/repl.rs drives a piped `--json` session through a success, a parse error, a command error (a token endpoint on a closed port) and a success again, and expects four JSON lines on stdout and no error on stderr. It fails against the stderr routing (two lines instead of four).

Breaking for a script reading a piped REPL's errors from stderr.

Spec updated: repl (MODIFIED: Interactive versus piped output).
