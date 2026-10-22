//! AC-14's eight subsystems and their derived rows (design §5.2, §3 rule 1).
//!
//! [`SubsystemHealth`] cannot be built with a word of the caller's choosing: the word is derived
//! from a reading by [`SubsystemHealth::from_measured_graded`] (or `from_belief`/`from_effect`),
//! so `ok` can never sit over an unknown (T-29, SEC-V4-25).

use std::time::{Instant, SystemTime};

use super::{BatteryInfo, DiskInfo, FirewallState, MemInfo};
use crate::core::measured::{Belief, Effect, Measured, Source, Unavailable};

/// The eight subsystems of AC-14, in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subsystem {
    Hardware,
    Controller,
    Cleaner,
    Security,
    Processes,
    Services,
    Network,
    IDevice,
}

impl Subsystem {
    pub const ALL: [Self; 8] = [
        Self::Hardware,
        Self::Controller,
        Self::Cleaner,
        Self::Security,
        Self::Processes,
        Self::Services,
        Self::Network,
        Self::IDevice,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Hardware => "Hardware",
            Self::Controller => "Controller",
            Self::Cleaner => "Cleaner",
            Self::Security => "Security",
            Self::Processes => "Processes",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::IDevice => "iDevice",
        }
    }
}

/// The only four words a health row may print (AC-14). Always painted as a word as well as in
/// colour, so no state is colour-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthWord {
    Ok,
    Attention,
    Unavailable,
    Off,
}

impl HealthWord {
    pub fn word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Attention => "attention",
            Self::Unavailable => "unavailable",
            Self::Off => "off",
        }
    }
}

/// How a live value becomes a word. A **function of the reading**, never a word the caller can
/// set, and only ever consulted when the reading is live.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grade {
    /// Any live reading is `ok`.
    Good,
    /// `attention` at or above the limit, `ok` below it.
    AtLeast(f64),
    /// `ok` when the switch really is on, `off` when it really is off.
    Switch,
}

/// One row of AC-14: `<subsystem> · <state> · <source>`, plus the reason when there is no state
/// to report.
///
/// Field privacy is the whole point: the old version had `pub word`, which is how T-29 painted a
/// green tick over a firewall it could not read.
#[derive(Clone, Debug)]
pub struct SubsystemHealth {
    subsystem: Subsystem,
    word: HealthWord,
    source: String,
    note: Option<String>,
}

impl SubsystemHealth {
    /// A row straight from a reading: `ok` while it is live, `unavailable` when it is missing,
    /// stale or failed — with the reason in `note`. No numeric bound: a row for a `bool` or for
    /// a reading that is not a number at all is still a row (Processes, Services, iDevice).
    pub fn from_measured<T>(subsystem: Subsystem, measured: &Measured<T>, now: Instant) -> Self {
        let word = if measured.observed(now).is_some() {
            HealthWord::Ok
        } else {
            HealthWord::Unavailable
        };
        Self::derived(subsystem, word, measured, now)
    }

    /// The same, but with the value mapping to `attention`/`off` at a threshold. `grade` is a
    /// [`Grade`], not a word: the word still falls out of the reading, so no caller can say
    /// `ok` over an `Unavailable` (SEC-V4-25).
    pub fn from_measured_graded<T: Copy + Into<f64>>(
        subsystem: Subsystem,
        measured: &Measured<T>,
        now: Instant,
        grade: Grade,
    ) -> Self {
        let word = match measured.observed(now) {
            Some(observed) => match grade {
                Grade::Good => HealthWord::Ok,
                Grade::AtLeast(limit) => {
                    let value: f64 = (*observed.value).into();
                    if value.is_finite() && value >= limit {
                        HealthWord::Attention
                    } else {
                        HealthWord::Ok
                    }
                }
                Grade::Switch => {
                    let value: f64 = (*observed.value).into();
                    if value > 0.5 {
                        HealthWord::Ok
                    } else {
                        HealthWord::Off
                    }
                }
            },
            None => HealthWord::Unavailable,
        };
        Self::derived(subsystem, word, measured, now)
    }

    /// The reading's provenance and its "why not" note, shared by both constructors so a row can
    /// never carry a word and a note that disagree about which reading they came from.
    fn derived<T>(subsystem: Subsystem, word: HealthWord, measured: &Measured<T>, now: Instant) -> Self {
        Self {
            subsystem,
            word,
            source: measured
                .provenance()
                .unwrap_or_else(|| "no source named".to_string()),
            note: Self::note_for(measured, now),
        }
    }

    /// A requested-only value is never a measurement: `Requested` is always `Unavailable` with
    /// the reason, never `ok` (SEC-V4-26).
    pub fn from_belief<T: Copy + Into<f64>>(
        subsystem: Subsystem,
        belief: &Belief<T>,
        source: &str,
        now: Instant,
    ) -> Self {
        match belief {
            Belief::Read(measured) => {
                let mut row = Self::from_measured(subsystem, measured, now);
                row.source = source.to_string();
                row
            }
            Belief::Requested { .. } => Self {
                subsystem,
                word: HealthWord::Unavailable,
                source: source.to_string(),
                note: Some(belief.render(now)),
            },
        }
    }

    /// An action's effect as a row: only `Confirmed` is `ok`; a request is `unavailable` with
    /// the words "not read back" (design §5.2).
    pub fn from_effect(subsystem: Subsystem, effect: &Effect, source: &str) -> Self {
        let (word, note) = match effect {
            Effect::Confirmed { observed } => (HealthWord::Ok, Some(observed.clone())),
            Effect::Requested { note, .. } => (
                HealthWord::Unavailable,
                Some(format!("requested, not read back — {note}")),
            ),
            Effect::Failed { message } => (HealthWord::Attention, Some(message.clone())),
            Effect::Refused { reason } => (HealthWord::Unavailable, Some(reason.clone())),
            Effect::Cancelled => (
                HealthWord::Unavailable,
                Some("cancelled — nothing decided".to_string()),
            ),
            Effect::Unavailable(reason) => (HealthWord::Unavailable, Some(reason.human())),
        };
        Self {
            subsystem,
            word,
            source: source.to_string(),
            note,
        }
    }

    fn note_for<T>(measured: &Measured<T>, now: Instant) -> Option<String> {
        if let Some(note) = measured.stale_note(now) {
            return Some(note);
        }
        measured.reason().map(Unavailable::human)
    }

    pub fn subsystem(&self) -> Subsystem {
        self.subsystem
    }

    pub fn word(&self) -> HealthWord {
        self.word
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// `"Hardware · ok · /sys/class/thermal"` — AC-14's row, words only (design §3 rule 1).
    pub fn words(&self) -> String {
        format!(
            "{} · {} · {}",
            self.subsystem.title(),
            self.word.word(),
            self.source
        )
    }
}

/// Everything Mission Control, the ribbon and the sidebar need, sampled at one moment.
///
/// There is deliberately no `health: [SubsystemHealth; 8]` field: the rows are derived on demand
/// by [`HealthSnapshot::health_rows`], so a word cannot contradict the reading that feeds it.
#[derive(Clone, Debug)]
pub struct HealthSnapshot {
    pub taken_at: Instant,
    /// The same moment on the wall clock, so the screen can print `Updated 09:41:07 · live`
    /// (design §3 rule 6) without a per-frame `SystemTime::now()` chain.
    pub taken_wall: SystemTime,
    pub cpu_temp_c: Measured<f32>,
    pub mem: Measured<MemInfo>,
    pub disk_root: Measured<DiskInfo>,
    pub battery: Measured<BatteryInfo>,
    pub fan: Measured<u32>,
    pub firewall: Measured<FirewallState>,
    /// `true` when the Controller Space's `start.sh` is there. Absence is stored as
    /// `Unavailable::Missing`, so this never holds `false` and an unknown can never read as
    /// "off".
    pub controller_installed: Measured<bool>,
}

impl HealthSnapshot {
    /// AC-14's eight rows, computed on demand from the readings above.
    pub fn health_rows(&self, now: Instant) -> [SubsystemHealth; 8] {
        let no_proc = Measured::<()>::unavailable(Unavailable::NotReported {
            source: Source::new("/proc", "process table"),
            why: "the process list arrives in the next update",
        });
        let no_services = Measured::<()>::unavailable(Unavailable::NotReported {
            source: Source::new("systemctl list-units", "failed units"),
            why: "the services list arrives in the next update",
        });
        let no_network = Measured::<()>::unavailable(Unavailable::NotReported {
            source: Source::new("ip route", "default route"),
            why: "the network checks arrive in the next update",
        });
        let no_device = Measured::<()>::unavailable(Unavailable::NotReported {
            source: Source::new("iDevice tab", "no device checked"),
            why: "the iDevice tab has not been opened this session",
        });

        [
            SubsystemHealth::from_measured_graded(
                Subsystem::Hardware,
                &self.cpu_temp_c,
                now,
                Grade::AtLeast(80.0),
            ),
            SubsystemHealth::from_measured(Subsystem::Controller, &self.controller_installed, now),
            SubsystemHealth::from_measured_graded(
                Subsystem::Cleaner,
                &self.disk_root,
                now,
                Grade::AtLeast(90.0),
            ),
            SubsystemHealth::from_measured_graded(
                Subsystem::Security,
                &self.firewall,
                now,
                Grade::Switch,
            ),
            SubsystemHealth::from_measured(Subsystem::Processes, &no_proc, now),
            SubsystemHealth::from_measured(Subsystem::Services, &no_services, now),
            SubsystemHealth::from_measured(Subsystem::Network, &no_network, now),
            SubsystemHealth::from_measured(Subsystem::IDevice, &no_device, now),
        ]
    }
}
