---
cairn: tasks
change: first-run-wizard-frame
---

# Tasks

- [x] Add a `configure` command (alias `wizard`) running the wizard by name, with no welcome and a non-interactive bail.
- [x] Move the welcome onto the offer, and give it the configuration path that was looked for.
- [x] Add `offer_configuration`, raised by a bare `ortie` and by `take_account`, returning whether the wizard ran.
- [x] Make a bare `ortie` show the help when a configuration exists, or when `--account` names one, instead of always running the wizard.
- [x] Make `take_account` a hook: offer, re-read the configuration, then fail the ordinary way when nothing landed.
- [x] Guard both entry points on `stdin().is_terminal()` and `printer.is_json()`.
- [x] Take the save path from `Config::target_path` instead of prompting for it, keeping print-then-save and the append confirmation.
- [x] Suffix the account name until free; claim `default` only when no other account does.
- [x] Name all three resolution failures: the path read, the accounts held, the two ways to pick a default.
- [x] Report where the account landed, under which name, and what to run next.
- [x] Read `ORTIE_CONFIG` like `-c`, both splitting on `:`.
- [x] Tests: a taken name gets a suffix, a missing configuration constrains nothing, an appended account keeps the existing one and its comments.
- [x] Build/test/fmt/clippy.
- [x] Fold into cairn/spec/config.md; log; land.
