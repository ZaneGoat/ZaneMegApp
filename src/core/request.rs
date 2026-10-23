//! What an action produced, and what was only asked for (design §3 rule 6, §5.1; SEC-V4-26).
//!
//! [`Effect`] is the outcome vocabulary — there is no `Ok(())` to mistake for a verified result —
//! and [`Belief`] is a value that was only ever *requested*. Split out of `measured.rs` for
//! AC-43's 500-line rule and re-exported from [`crate::core::measured`], which is the path
//! callers use. Slice 4a defines the vocabulary; 4b is its first writer (§13.8).

use std::time::{Instant, SystemTime};

use crate::core::measured::Measured;
use crate::core::source::Unavailable;
use crate::core::time;

/// What an action produced.
///
/// There is deliberately no `Ok(())`: a caller that has not read the state back must return
/// [`Effect::Requested`], and the UI words it differently from [`Effect::Confirmed`]
/// (SEC-V4-26, design §5.1). Slice 4b constructs these from `CoreEvent::ActionFinished`; 4a
/// defines the vocabulary (§13.8), so nothing here is written by the UI yet.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // 4b is the first writer (design §13.8: "4a defines, 4b uses").
pub enum Effect {
    /// The state was read back and it matched what was asked for.
    Confirmed { observed: String },
    /// The command was issued; the state was **not** read back.
    Requested {
        observed: Option<String>,
        note: String,
    },
    /// The tool ran and failed. `message` is its own first line, escaped.
    Failed { message: String },
    /// Refused before running: protected, invalid, or test mode.
    Refused { reason: String },
    Cancelled,
    Unavailable(Unavailable),
}

#[allow(dead_code)] // See `Effect`: 4b reads these words; 4a fixes the vocabulary.
impl Effect {
    /// The one word the activity list and a toast may print. Never "done" for a request.
    pub fn word(&self) -> &'static str {
        match self {
            Self::Confirmed { .. } => "confirmed",
            Self::Requested { .. } => "requested",
            Self::Failed { .. } => "failed",
            Self::Refused { .. } => "refused",
            Self::Cancelled => "cancelled",
            Self::Unavailable(_) => "unavailable",
        }
    }

    /// Only a read-back may be reported as success (SEC-V4-26).
    pub fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed { .. })
    }
}

/// A value that is only ever *requested* — Cooler Boost, a power profile, a battery charge
/// limit — is a belief, never a measurement (SEC-V4-26, design §5.1).
#[derive(Clone, Debug)]
pub enum Belief<T> {
    /// The value is read back from the platform: this half *is* a measurement.
    Read(Measured<T>),
    /// She asked for it at `at`; nothing has read it back.
    Requested {
        value: T,
        at: SystemTime,
        note: &'static str,
    },
}

impl<T: Copy + Into<f64>> Belief<T> {
    /// Always carries "last requested … — not read back" for a request, and never a unit: a
    /// requested value is not a measurement and must not look like one.
    pub fn render(&self, now: Instant) -> String {
        match self {
            Self::Requested { value, at, note } => {
                let number = (*value).into();
                format!(
                    "last requested {number:.0} at {} — not read back ({note})",
                    time::hh_mm(*at)
                )
            }
            Self::Read(measured) => {
                if let Some(observed) = measured.observed(now) {
                    let number: f64 = (*observed.value).into();
                    format!(
                        "{number:.0} — read back from {}",
                        observed.source.what
                    )
                } else if let Some(note) = measured.stale_note(now) {
                    note
                } else {
                    measured.reason().map_or_else(
                        || "unavailable — not read back".to_string(),
                        |reason| format!("unavailable — {}", reason.human()),
                    )
                }
            }
        }
    }

    pub fn is_requested(&self) -> bool {
        matches!(self, Self::Requested { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::measured::Unit;
    use crate::core::source::Source;
    use std::time::{Duration, UNIX_EPOCH};

    fn wall() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }

    /// A requested value is worded differently from a measurement, and never carries a unit.
    #[test]
    fn a_requested_belief_never_reads_back() {
        let now = Instant::now();
        let requested: Belief<u32> = Belief::Requested {
            value: 80,
            at: wall(),
            note: "the firmware does not confirm it",
        };
        let line = requested.render(now);
        assert!(line.contains("last requested"), "{line}");
        assert!(line.contains("not read back"), "{line}");
        for unit in [Unit::Celsius, Unit::Percent, Unit::GiB, Unit::Rpm] {
            assert!(
                !line.contains(unit.suffix()),
                "a request must not carry {}: {line}",
                unit.suffix()
            );
        }
        assert!(requested.is_requested());

        let unread: Belief<u32> = Belief::Read(Measured::unavailable(Unavailable::NotReported {
            source: Source::new("/sys/class/power_supply/BAT1", "charge_control_end_threshold"),
            why: "this firmware does not report the charge limit",
        }));
        let line = unread.render(now);
        assert!(line.starts_with("unavailable — "), "{line}");
        assert!(!unread.is_requested());
    }

    /// One vocabulary for actions, and no `done` for a request (design §3 rule 6).
    #[test]
    fn an_action_says_what_was_observed() {
        let confirmed = Effect::Confirmed {
            observed: "Active: inactive (dead)".into(),
        };
        assert_eq!(confirmed.word(), "confirmed");
        assert!(confirmed.is_confirmed());

        let requested = Effect::Requested {
            observed: None,
            note: "issued; not read back".into(),
        };
        assert_eq!(requested.word(), "requested");
        assert!(!requested.is_confirmed(), "a request is not a success");
        assert_ne!(requested.word(), "done");

        for (effect, word) in [
            (
                Effect::Failed {
                    message: "Failed to connect to bus".into(),
                },
                "failed",
            ),
            (
                Effect::Refused {
                    reason: "protected service".into(),
                },
                "refused",
            ),
            (Effect::Cancelled, "cancelled"),
            (Effect::Unavailable(Unavailable::TestMode), "unavailable"),
        ] {
            assert_eq!(effect.word(), word);
            assert!(!effect.is_confirmed());
        }
    }
}
