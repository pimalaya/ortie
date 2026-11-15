---
cairn: change
id: repl-errors-on-stdout
status: landed
created: 2026-10-27
---

# A piped REPL reports errors on stdout

## Why

The family rule (pimalaya-cli spec/error.md, cairn/spec/output.md here) is that data and errors both go to stdout through the printer, distinguished by the exit code or the object shape, and that stderr carries logs and prompts only. A one-shot `ortie` already follows it: `ErrorReport::eval` prints the failure on stdout.

The REPL did not. When piped it wrote a command error as `{err:?}` and a parse error as clap's message on stderr, so an application driving it had to read two streams, could not tell which stdout line a failure belonged to, and under `--json` got no `ortie-error` object at all.

## What

When stdin is not a TTY, a parse error or a command error is printed through the printer as an `ErrorReport`, on stdout, flushed, and the loop continues. Under `--json` that is one `ortie-error` object per line, among the payload objects of the commands that succeeded, so a driver reads one stream and tells a failure apart by its `error` key.

Help and version requests (`token --help`) are not failures and keep clap's own stdout output. The interactive TTY session is unchanged: its prompt and its errors stay on stderr for a human reading the terminal.

This breaks a script reading a piped REPL's errors from stderr.
