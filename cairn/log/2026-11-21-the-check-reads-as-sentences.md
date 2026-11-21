---
cairn: log
change: the-check-reads-as-sentences
landed: 2026-11-21
---

# The check reads as sentences

Owner ruling of 2026-11-21: `account check` no longer prints its rows twice on a terminal. Its text events read as sentences naming what each check does, `Reading the token storage… OK`, `Checking the stored token… OK`, `Reaching the token endpoint… OK, TLS`, the last one followed by a blank line; its text report counts the checks, `All 5 checks of <account> passed`, or `<f> of <n> checks of <account> failed:` then each failed row and why. The failed rows stay in the report because a redirected text output carries no event, so a log or a cron mail keeps why a check failed. The JSON events and result are unchanged.

## What landed

- src/account/check.rs: `action` names each check; `AccountCheckEvent::CheckFinished` carries `last`, skipped from the JSON and its schema, ending the last event on a blank line; `AccountCheckOutput`'s text is the count then the failed rows.
- tests/account.rs: the text reports asserted in their new form; the TLS text, now printed by the events alone, is covered by the unit test `an_event_prints_one_sentence`.

## Spec

account ("The check proves the storage, the token and the endpoints": the text event and report wording).

## Verified

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (139 passed), and a run on a pseudo-terminal against the Fastmail test account and a scratch account on a closed port.
