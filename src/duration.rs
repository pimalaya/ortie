//! # Duration
//!
//! The text form of a duration Ortie prints: a token's age and lifetime,
//! a device code's lifetime. Written by hand in the form humantime gave
//! (`1h 30m`, `2days 4h`), which the text output kept when the crate left.

use std::{fmt, time::Duration};

/// Seconds in a year as humantime counted it, 365.25 days.
const YEAR_SECS: u64 = 31_557_600;

/// Seconds in a month as humantime counted it, 30.44 days.
const MONTH_SECS: u64 = 2_630_016;

/// Seconds in a day.
const DAY_SECS: u64 = 86_400;

/// A duration printed in whole units, largest first: years, months and
/// days pluralized (`2days`), then `h`, `m` and `s`, `0s` when empty.
///
/// The fraction of a second is dropped: Ortie counts in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HumanDuration(pub Duration);

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();

        if secs == 0 {
            return f.write_str("0s");
        }

        let rest = secs % YEAR_SECS % MONTH_SECS % DAY_SECS;

        let units = [
            (secs / YEAR_SECS, "year", true),
            (secs % YEAR_SECS / MONTH_SECS, "month", true),
            (secs % YEAR_SECS % MONTH_SECS / DAY_SECS, "day", true),
            (rest / 3600, "h", false),
            (rest % 3600 / 60, "m", false),
            (rest % 60, "s", false),
        ];

        let mut started = false;

        for (value, name, plural) in units {
            if value == 0 {
                continue;
            }

            if started {
                f.write_str(" ")?;
            }

            write!(f, "{value}{name}")?;

            if plural && value > 1 {
                f.write_str("s")?;
            }

            started = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::duration::HumanDuration;

    /// The form humantime 2.4 printed, kept by the text output.
    #[test]
    fn a_duration_prints_as_humantime_did() {
        let cases = [
            (0, "0s"),
            (59, "59s"),
            (3600, "1h"),
            (3601, "1h 1s"),
            (5400, "1h 30m"),
            (86_400, "1day"),
            (2 * 86_400 + 4 * 3600, "2days 4h"),
            (2_630_016, "1month"),
            (
                31_557_600 * 2 + 2_630_016 + 86_400 + 61,
                "2years 1month 1day 1m 1s",
            ),
        ];

        for (secs, text) in cases {
            let printed = HumanDuration(Duration::from_secs(secs)).to_string();
            assert_eq!(printed, text, "{secs}");
        }
    }
}
