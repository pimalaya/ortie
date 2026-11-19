---
cairn: log
change: durations-in-jiffs-friendly-form
landed: 2026-11-19
---

# Durations in jiff's friendly form

Owner ruling of 2026-11-19. src/duration.rs (`HumanDuration`, humantime's look written by hand) and its test `a_duration_prints_as_humantime_did` are deleted. src/token/issued.rs `friendly` prints a duration in seconds as a jiff 1.0.1 `Span` balanced up to days of 24 hours (`SpanRound::largest(Unit::Day).days_are_24_hours()`), through the default friendly printer: `90` reads `1m 30s`, `5400` `1h 30m`, `187200` `2d 4h` (was `2days 4h`). A span holds at most about 20,000 years; past it the duration prints in hours through `SpanPrinter::unsigned_duration_to_string`. `token inspect`, the issued-token receipt and the `auth get` device request call it.

Evidence: the unit test `a_duration_prints_in_days_at_most` (`0s`, `1m 30s`, `1h 30m`, `2d 4h`, `400d 1m 1s`, and `u64::MAX` in hours). tests/token_inspect.rs asserts `in 49m`, which both forms print, and needed no change. The JSON was not touched: `expiresIn` stays seconds, `issuedAt` and `expiresAt` instants.

Capabilities moved: output (the requirement "A duration prints in jiff's friendly form").

`cargo fmt`, `cargo test` and `cargo clippy --all-targets -- -D warnings` (default and `--all-features`) pass; `cargo deny check`: advisories, bans, licenses and sources ok.
