//! `Measured<T>` — a number that cannot be separated from where it came from and when.
//!
//! This is the type that makes design §3 rule 1 ("every number names its source and its unit, or
//! says `unavailable` and why") true by construction, not by habit (AC-07, AC-11, AC-12, AC-14;
//! SEC-V4-25, design §5.1, §13.3/13.8):
//!
//! * **No public field, no getter, no `Deref` yields a bare `T`.** The only way out is
//!   [`Measured::observed`], which hands back the value *together with* its [`Source`] and its
//!   age — so a caller cannot keep the number and drop the provenance.
//! * **The budget belongs to the reading.** [`Cadence`] is 3 × the source's cadence, floored at
//!   2 s and capped at 60 s, computed inside the constructor. There is no argument through which
//!   a caller can ask for a longer one, and no `is_live()` flag to consult instead of the age:
//!   past the budget [`Measured::observed`] returns `None` and closes itself.
//! * **Unknown is its own state.** [`Unavailable`] carries where the read was attempted and why
//!   it failed, and `render()` returns `None` for it — an unknown can never be formatted as `0`,
//!   `NaN` or an empty cell.
//!
//! The provenance types ([`Source`], [`Unavailable`]) live in `source.rs` and the action half of
//! the vocabulary ([`Effect`], [`Belief`]) in `request.rs`; both are re-exported here, so
//! `crate::core::measured::…` stays the single import a caller needs. The split is AC-43's
//! 500-line rule and nothing else.

use std::time::{Duration, Instant, SystemTime};

use crate::core::time;

pub use crate::core::request::{Belief, Effect};
pub use crate::core::source::{Source, Unavailable};

/// The freshness budget never drops below this: two seconds of grace so a single slow tick
/// cannot turn a good reading into an unavailable one.
const BUDGET_FLOOR: Duration = Duration::from_secs(2);
/// …and never rises above this, so a 0.1 Hz source is re-read long before it is a minute old.
const BUDGET_CAP: Duration = Duration::from_secs(60);
/// The budget is this many source periods.
const BUDGET_PERIODS: u32 = 3;

/// How often a source is read, supplied where the reading is made (the sampler).
///
/// The freshness budget is derived here and nowhere else: a caller chooses a *cadence*, which is
/// a property of the source, and cannot choose the budget, which is the guarantee. A 1 Hz CPU
/// temperature is stale after 3 s; a 0.1 Hz `statvfs` read after 30 s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cadence(Duration);

impl Cadence {
    pub const fn every(period: Duration) -> Self {
        Self(period)
    }

    /// `3 × period`, floor [`BUDGET_FLOOR`], cap [`BUDGET_CAP`].
    pub fn budget(self) -> Duration {
        self.0
            .saturating_mul(BUDGET_PERIODS)
            .clamp(BUDGET_FLOOR, BUDGET_CAP)
    }
}

#[derive(Clone, Debug)]
pub struct Measured<T> {
    state: State<T>,
}

#[derive(Clone, Debug)]
enum State<T> {
    Read(Reading<T>),
    Unavailable(Unavailable),
}

#[derive(Clone, Debug)]
struct Reading<T> {
    value: T,
    source: Source,
    /// Monotonic — the age.
    read_at: Instant,
    /// Wall clock — the "last read 09:41:07" a person reads.
    read_wall: SystemTime,
    /// The freshness budget this reading owns. Fixed at construction (§5.1).
    budget: Duration,
}

/// A live reading plus everything needed to interpret it.
///
/// Obtainable only from [`Measured::observed`]: `_seal` is private, so a struct literal outside
/// this module does not compile — which is what stops a `Measured`-shaped value being forged.
#[derive(Debug, PartialEq, Eq)]
pub struct Observed<'a, T> {
    pub value: &'a T,
    pub source: &'a Source,
    pub age: Duration,
    pub budget: Duration,
    _seal: (),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueStatus {
    /// Read, and inside its own budget.
    Live,
    /// Read, but older than the budget it owns — the cell says when, not what.
    Stale,
    /// Never read, or read and failed.
    Unavailable,
}

impl ValueStatus {
    pub fn word(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

/// The unit a number carries on screen. A value is only ever printed with one of these, so
/// "8.4" is never mistaken for "8.4 GiB".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Celsius,
    Percent,
    GiB,
    MiB,
    Rpm,
    Watts,
    RttMs,
    Seconds,
    /// A bare count with the noun that makes it readable (`"3 units"`).
    Count(&'static str),
}

impl Unit {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Celsius => "°C",
            Self::Percent => "%",
            Self::GiB => "GiB",
            Self::MiB => "MiB",
            Self::Rpm => "rpm",
            Self::Watts => "W",
            Self::RttMs => "ms",
            Self::Seconds => "s",
            Self::Count(word) => word,
        }
    }
}

impl<T> Measured<T> {
    /// Crate-internal, and the only way to make a reading: a value arrives with its source, its
    /// time and the cadence it will be re-read at — always together, never one at a time.
    pub(crate) fn read(
        value: T,
        source: Source,
        at: Instant,
        wall: SystemTime,
        cadence: Cadence,
    ) -> Self {
        Self {
            state: State::Read(Reading {
                value,
                source,
                read_at: at,
                read_wall: wall,
                budget: cadence.budget(),
            }),
        }
    }

    /// A reading that did not happen, with the reason it did not.
    pub fn unavailable(reason: Unavailable) -> Self {
        Self {
            state: State::Unavailable(reason),
        }
    }

    /// The only accessor that yields the number, and it yields the provenance with it.
    ///
    /// `None` when the reading is stale or absent, so no caller — UI or business logic — can
    /// consume an old value as if it were current.
    pub fn observed(&self, now: Instant) -> Option<Observed<'_, T>> {
        match &self.state {
            State::Read(reading) => {
                let age = now.saturating_duration_since(reading.read_at);
                (age <= reading.budget).then(|| Observed {
                    value: &reading.value,
                    source: &reading.source,
                    age,
                    budget: reading.budget,
                    _seal: (),
                })
            }
            State::Unavailable(_) => None,
        }
    }

    pub fn status(&self, now: Instant) -> ValueStatus {
        match &self.state {
            State::Read(reading) => {
                if now.saturating_duration_since(reading.read_at) <= reading.budget {
                    ValueStatus::Live
                } else {
                    ValueStatus::Stale
                }
            }
            State::Unavailable(_) => ValueStatus::Unavailable,
        }
    }

    pub fn source(&self) -> Option<&Source> {
        match &self.state {
            State::Read(reading) => Some(&reading.source),
            State::Unavailable(_) => None,
        }
    }

    pub fn age(&self, now: Instant) -> Option<Duration> {
        match &self.state {
            State::Read(reading) => Some(now.saturating_duration_since(reading.read_at)),
            State::Unavailable(_) => None,
        }
    }

    pub fn reason(&self) -> Option<&Unavailable> {
        match &self.state {
            State::Read(_) => None,
            State::Unavailable(reason) => Some(reason),
        }
    }

    /// Where this value came from, or where the failed read was attempted — so even an
    /// `unavailable` cell can name its source (§3 rule 1).
    pub fn provenance(&self) -> Option<String> {
        self.source()
            .map(Source::words)
            .or_else(|| self.reason().and_then(Unavailable::source).map(Source::words))
    }

    /// `Some("stale — last read 09:41:07")` when the reading is past its budget, so a cell can
    /// still say something true instead of showing an old number or nothing at all.
    pub fn stale_note(&self, now: Instant) -> Option<String> {
        match &self.state {
            State::Read(reading)
                if now.saturating_duration_since(reading.read_at) > reading.budget =>
            {
                Some(format!("stale — last read {}", time::hh_mm_ss(reading.read_wall)))
            }
            _ => None,
        }
    }
}

impl<T: Copy + Into<f64>> Measured<T> {
    /// `Some("75 °C")` when live; `None` otherwise.
    ///
    /// `None` for an `Unavailable` **and** for every stale reading: a value the type has stopped
    /// vouching for cannot be formatted, let alone as `"0"`. A non-finite number is not a
    /// measurement either, so it is refused here too.
    pub fn render(&self, now: Instant, unit: Unit, decimals: u8) -> Option<String> {
        let observed = self.observed(now)?;
        let number: f64 = (*observed.value).into();
        if !number.is_finite() {
            return None;
        }
        Some(format_number(number, unit, decimals))
    }
}

/// `75.0` + [`Unit::Celsius`] + 0 decimals → `"75 °C"`.
pub fn format_number(number: f64, unit: Unit, decimals: u8) -> String {
    let suffix = unit.suffix();
    if suffix.is_empty() {
        format!("{number:.*}", usize::from(decimals))
    } else {
        format!("{number:.*} {suffix}", usize::from(decimals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    fn wall() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }

    fn meminfo() -> Measured<f32> {
        Measured::read(
            54.0,
            Source::new("/proc/meminfo", "MemTotal/MemAvailable"),
            Instant::now(),
            wall(),
            Cadence::every(Duration::from_secs(1)),
        )
    }

    /// A fresh reading reads, and it reads *with* its source and its age — §3 rule 1's half that
    /// no caller can drop.
    #[test]
    fn a_fresh_value_reads_with_its_source() {
        let now = Instant::now();
        let m = meminfo();
        let observed = m.observed(now).expect("a fresh reading is observable");
        assert_eq!(*observed.value, 54.0);
        assert_eq!(observed.source.what, "/proc/meminfo");
        assert_eq!(observed.source.words(), "/proc/meminfo — MemTotal/MemAvailable");
        assert!(observed.age < Duration::from_secs(1));
        assert_eq!(observed.budget, Duration::from_secs(3));
        assert_eq!(m.status(now), ValueStatus::Live);
        assert_eq!(m.render(now, Unit::Percent, 0).as_deref(), Some("54 %"));
        assert_eq!(m.stale_note(now), None);
    }

    /// One nanosecond past the budget the door closes, and the cell still says something true.
    #[test]
    fn a_stale_value_does_not_read_and_says_when() {
        let read_at = Instant::now();
        let m = Measured::read(
            75.0,
            Source::new("/sys/class/thermal", "thermal_zone0"),
            read_at,
            wall(),
            Cadence::every(Duration::from_secs(1)),
        );
        let just_inside = read_at + Duration::from_secs(3);
        assert!(m.observed(just_inside).is_some(), "age == budget is still live");

        let just_outside = just_inside + Duration::from_nanos(1);
        assert!(m.observed(just_outside).is_none());
        assert_eq!(m.status(just_outside), ValueStatus::Stale);
        assert_eq!(
            m.render(just_outside, Unit::Celsius, 0),
            None,
            "a stale reading must not format"
        );
        let note = m.stale_note(just_outside).expect("a stale note");
        assert!(note.starts_with("stale — last read "), "{note}");
        assert_eq!(note.len(), "stale — last read ".len() + 8, "{note}");
    }

    /// The budget is 3 × cadence, floored at 2 s and capped at 60 s — and nothing else.
    #[test]
    fn the_budget_follows_the_cadence_and_cannot_be_widened() {
        let sec = Duration::from_secs(1);
        assert_eq!(Cadence::every(sec).budget(), Duration::from_secs(3));
        assert_eq!(Cadence::every(Duration::from_millis(10)).budget(), BUDGET_FLOOR);
        assert_eq!(Cadence::every(Duration::from_millis(500)).budget(), BUDGET_FLOOR);
        assert_eq!(
            Cadence::every(Duration::from_secs(10)).budget(),
            Duration::from_secs(30)
        );
        assert_eq!(
            Cadence::every(Duration::from_secs(60)).budget(),
            Duration::from_secs(60)
        );
        assert_eq!(
            Cadence::every(Duration::from_secs(3_600)).budget(),
            BUDGET_CAP
        );

        // …and a reading owns the budget it was given: the only constructor takes a cadence.
        let read_at = Instant::now();
        let hourly = Measured::read(
            1.0,
            Source::new("/proc/meminfo", ""),
            read_at,
            wall(),
            Cadence::every(Duration::from_secs(3_600)),
        );
        assert_eq!(
            hourly.observed(read_at + Duration::from_secs(61)),
            None,
            "a 60 s cap is a cap, not a suggestion"
        );
    }

    /// Unknown is its own state for every reason there is, and it never formats as a number.
    #[test]
    fn unknown_never_renders_as_a_zero() {
        let source = Source::new("/proc/meminfo", "MemTotal/MemAvailable");
        let reasons = [
            Unavailable::NotChecked,
            Unavailable::Missing {
                source: source.clone(),
            },
            Unavailable::Denied {
                source: source.clone(),
            },
            Unavailable::Parse {
                source: source.clone(),
                detail: "no MemTotal".into(),
            },
            Unavailable::Command {
                program: "systemctl",
                exit: Some(1),
                message: "Failed to connect to bus".into(),
            },
            Unavailable::Timeout {
                program: "ping",
                after: Duration::from_secs(5),
            },
            Unavailable::NotReported {
                source: Source::new("/sys/class/hwmon", "no fan*_input"),
                why: "this firmware does not report fan speed",
            },
            Unavailable::ProgramMissing { path: "/usr/local/bin/isw" },
            Unavailable::TestMode,
        ];
        let now = Instant::now();
        for reason in reasons {
            let expected = reason.human();
            assert!(!expected.is_empty(), "{reason:?} has no sentence");
            let m: Measured<f32> = Measured::unavailable(reason.clone());
            assert!(m.observed(now).is_none(), "{reason:?} must not be observable");
            assert_eq!(m.status(now), ValueStatus::Unavailable);
            assert_eq!(m.render(now, Unit::Percent, 1), None, "{reason:?} formatted");
            assert_eq!(m.stale_note(now), None, "{reason:?} is not stale, it is unknown");
            assert_eq!(m.reason(), Some(&reason));
            // The word on screen is the reason, never a number and never "off" (T-29).
            let word = m.reason().map(Unavailable::human).expect("a reason");
            assert!(!word.starts_with('0'), "{word}");
            assert!(!word.contains("NaN"), "{word}");
            assert_ne!(word, "off", "{word}");
        }
    }

    /// A non-finite reading is not a measurement: it does not format as `NaN`.
    #[test]
    fn a_nan_reading_does_not_format() {
        let now = Instant::now();
        for value in [f32::NAN, f32::INFINITY] {
            let m = Measured::read(
                value,
                Source::new("/sys/class/thermal", "thermal_zone0"),
                now,
                wall(),
                Cadence::every(Duration::from_secs(1)),
            );
            assert_eq!(m.render(now, Unit::Celsius, 0), None);
        }
    }

    /// The source text survives every path through the type, including a failed read.
    #[test]
    fn the_source_text_survives() {
        let now = Instant::now();
        let source = Source::new("/sys/class/power_supply/BAT1", "capacity");
        let live = Measured::read(79_u32, source.clone(), now, wall(), Cadence::every(Duration::from_secs(1)));
        assert_eq!(live.provenance().as_deref(), Some("/sys/class/power_supply/BAT1 — capacity"));
        assert_eq!(source.read_from(), "read from /sys/class/power_supply/BAT1");

        let failed: Measured<u32> = Measured::unavailable(Unavailable::Denied { source });
        assert_eq!(
            failed.provenance().as_deref(),
            Some("/sys/class/power_supply/BAT1 — capacity"),
            "even an unreadable source names itself"
        );
        assert_eq!(failed.render(now, Unit::Percent, 0), None);
    }

    /// The status words are the ones AC-14 and §3 rule 6 fix, and are not the empty string.
    #[test]
    fn status_words_are_the_three_ac14_words() {
        assert_eq!(ValueStatus::Live.word(), "live");
        assert_eq!(ValueStatus::Stale.word(), "stale");
        assert_eq!(ValueStatus::Unavailable.word(), "unavailable");
    }
}
