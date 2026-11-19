---
cairn: change
id: jiff-replaces-humantime
status: landed
created: 2026-11-19
---

# jiff replaces humantime

## Why

`cargo deny check` fails on RUSTSEC-2026-0542: humantime is unmaintained. Owner ruling of 2026-11-19: swap it for jiff, as io-pimdir did in 0.16 (its duration flags). Ortie used humantime for two outputs: the RFC 3339 instants of `token inspect --json` and the durations of the text output.

## What

- The instants are written through jiff's civil date-time in UTC, `Z` appended, so the clamp to 9999-12-31T23:59:59Z still prints (jiff's `Timestamp` stops short of it).
- The durations keep humantime's form (`1h 30m`, `2days 4h`): no other application of the family prints a duration through jiff, so there is no family form to align on, and the text stays as it was. A small formatter, src/duration.rs, writes it.

### Breaking

None: both outputs are byte for byte what humantime printed.
