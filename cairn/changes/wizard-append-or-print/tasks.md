---
cairn: tasks
change: wizard-append-or-print
---

# Tasks

- [x] Split `offer_save` into `save_or_print`, `append_or_print` and `print_saved`, with the reference's wording.
- [x] Short-circuit on `printer.is_json() || !stdout().is_terminal()`, printing the account and touching no file.
- [x] Branch the terminal path on whether a configuration file is already there, one prompt each.
- [x] Keep the append a plain text append, through `OpenOptions::new().append(true)`.
- [x] Drop the `--json` exemption from the stdin guard.
- [x] Update the module, command and function docs the reordering made wrong.
- [x] Changelog: Changed for the save step, Fixed for the stdin guard.
- [x] Build, test, fmt and clippy clean; the non-interactive paths touch no file and do not hang.
- [x] Fold into cairn/spec/discovery.md and cairn/spec/config.md; log; land.
