---
cairn: log
change: jiff-replaces-humantime
landed: 2026-11-19
---

# jiff replaces humantime

Owner ruling of 2026-11-19, after RUSTSEC-2026-0542 (humantime unmaintained). Cargo.toml depends on jiff 1 (`std` only, no time zone database) instead of humantime 2, which leaves the lock; jiff 1.0.1 brings jiff-core, jiff-static and portable-atomic.

- src/token/inspect.rs writes `issuedAt` and `expiresAt` from jiff's civil date-time at the Unix epoch plus the stored seconds, `Z` appended. jiff's `Timestamp` cannot hold 9999-12-31T23:59:59Z, the instant an endless lifetime clamps to, a civil date-time can. tests/token_inspect.rs now builds its expected instant with `jiff::Timestamp`.
- src/duration.rs `HumanDuration` prints a duration as humantime 2.4 did (years of 365.25 days, months of 30.44 days, days pluralized, then `h`, `m`, `s`, `0s` when empty), for `token inspect`, the issued-token receipt and the device request. No application of the family prints a duration through jiff (io-pimdir parses its duration flags with jiff; no repository of the seven prints one through it), so the form stays. Before humantime left, a temporary test compared both on every second below 200,000 and on a geometric walk to 200,000,000: identical. The kept unit test `a_duration_prints_as_humantime_did` pins the form.

`cargo deny check`: advisories, bans, licenses and sources ok; the two warnings left were there before (the getrandom 0.2 duplicate ring needs, an allowed GitHub organization no source uses).

Capabilities moved: none, the outputs being unchanged.
