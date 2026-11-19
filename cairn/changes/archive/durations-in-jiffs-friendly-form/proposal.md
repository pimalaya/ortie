---
cairn: change
id: durations-in-jiffs-friendly-form
status: landed
created: 2026-11-19
---

# Durations in jiff's friendly form

## Why

Owner ruling of 2026-11-19, reversing the part of jiff-replaces-humantime that kept humantime's look: the text output prints its durations as jiff prints them, and src/duration.rs, a formatter written by hand to imitate a crate that left, goes.

## What

- A duration of the text output (the `Issued` age and the `Expires` lifetime of `token inspect`, the issued-token receipt, the `Expires in` field of the `auth get` device request) is a jiff `Span` of whole seconds, rounded with `largest(Unit::Day)` and `days_are_24_hours()`, printed with the `{:#}` alternate `Display`, the default friendly printer.
- That printer's compact designators (`d`, `h`, `m`, `s`) and spaces between units: `1m 30s`, `1h 30m`, `2d 4h`. Short as humantime's and with one spelling per unit, no plural. The verbose (`2 days 4 hours`) and the short (`2 days 4 hrs`) designators read better but double the width of an aligned field.
- Days are the largest unit, each of 24 hours: a lifetime counted in seconds has no calendar behind it, so weeks, months and years, which humantime gave as 30.44 and 365.25 days, would be guesses. jiff's `SignedDuration` printing stops at hours (`52h`), too coarse for a 90-day refresh lifetime.
- A duration past a jiff span's range (about 20,000 years) prints through `SpanPrinter::unsigned_duration_to_string`, in hours.
- The JSON prints seconds (`expiresIn`) and instants (`issuedAt`, `expiresAt`): unchanged.

### Breaking

The text of `token inspect`, of the receipt and of the device request: `2d 4h` where `2days 4h` was printed, `400d 1m 1s` where `1year 1month 4days 7h 27m 25s` was. At ortie's next major (v3), in the CHANGELOG and MIGRATION.md.
