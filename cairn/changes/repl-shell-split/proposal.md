---
cairn: change
id: repl-shell-split
status: landed
created: 2026-11-04
---

# REPL lines split as a shell splits them

## Why

`ortie repl` split each line on whitespace, so a quoted word kept its quotes (`token "show"` was an unknown subcommand) and a value could never carry a space. himalaya's REPL splits with `shlex` (himalaya cairn/spec/repl.md), and the two sessions should read a line the same way.

## What

Each line is split with `shlex::split`, quotes and backslash escapes included. A line whose quote is left open fails with "Cannot split the line: a quote is not closed", reported like any command error, and the loop continues.
