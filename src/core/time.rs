//! The one clock the screen prints (design §4, `core/time.rs`).
//!
//! Everything she reads a time from — `stale — last read 09:41:07`, an activity line, a
//! request's "last requested ON at 09:41" — must agree with the clock in the corner of her
//! screen. That is *local* time, so this module asks the C library for it instead of doing UTC
//! arithmetic in a `const fn`. `localtime_r` is the single `unsafe` in the file and it is
//! confined to [`local_parts`], which falls back to UTC if it ever returns null.

use std::time::{SystemTime, UNIX_EPOCH};

/// `HH:MM:SS`, local time — the form the stale note and the activity list use (design §3 rule 6).
pub fn hh_mm_ss(at: SystemTime) -> String {
    let (hour, minute, second) = local_parts(at);
    format!("{hour:02}:{minute:02}:{second:02}")
}

/// `HH:MM`, local time — the shorter form a sentence uses ("last requested ON at 09:41").
pub fn hh_mm(at: SystemTime) -> String {
    let (hour, minute, _) = local_parts(at);
    format!("{hour:02}:{minute:02}")
}

/// The local wall-clock time, or UTC when the C library cannot work it out.
fn local_parts(at: SystemTime) -> (i64, i64, i64) {
    let secs = at.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: `tm` is a zeroed, correctly sized `libc::tm` owned by this function; `localtime_r`
    // writes into it and returns either that pointer or null, which we check before reading.
    let ok = unsafe { libc::localtime_r(&secs, &mut tm) };
    if ok.is_null() {
        let day = secs.rem_euclid(86_400);
        return (day / 3_600, (day % 3_600) / 60, day % 60);
    }
    (tm.tm_hour as i64, tm.tm_min as i64, tm.tm_sec as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    /// Both forms are fixed-width clock faces: no `-1`, no decimals, nothing to misread.
    #[test]
    fn a_time_is_two_digits_per_field() {
        for secs in [0, 1_700_000_000, 951_782_400] {
            let long = hh_mm_ss(at(secs));
            let short = hh_mm(at(secs));
            assert_eq!(long.len(), 8, "{long}");
            assert_eq!(short.len(), 5, "{short}");
            assert!(long.starts_with(&short), "{long} vs {short}");
            for field in long.split(':').chain(short.split(':')) {
                assert_eq!(field.len(), 2, "{long} / {short}");
                assert!(field.parse::<u32>().is_ok(), "{long} / {short}");
            }
        }
    }

    /// One second later is one second later, whatever the zone offset is.
    #[test]
    fn consecutive_seconds_are_consecutive_on_the_clock() {
        let first = hh_mm_ss(at(1_700_000_000)).replace(':', "").parse::<u32>();
        let second = hh_mm_ss(at(1_700_000_001)).replace(':', "").parse::<u32>();
        let (first, second) = (first.expect("digits"), second.expect("digits"));
        // 23:59:59 -> 00:00:00 wraps; every other step is exactly +1 on a 6-digit field.
        let step = (second + 240_000 - first) % 240_000;
        assert_eq!(step, 1, "{} -> {}", hh_mm_ss(at(1_700_000_000)), hh_mm_ss(at(1_700_000_001)));
    }

    /// A time before the epoch is still a clock face, not a negative number.
    #[test]
    fn a_pre_epoch_time_does_not_panic() {
        let before = UNIX_EPOCH - Duration::from_secs(60);
        assert_eq!(hh_mm_ss(before).len(), 8);
    }
}
