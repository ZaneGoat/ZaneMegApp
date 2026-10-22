//! Where a reading came from, and why there is none (design §3 rule 1, §5.1; AC-07, AC-11).
//!
//! Split out of `measured.rs` for AC-43's 500-line rule and re-exported from
//! [`crate::core::measured`], which is the path callers use.

use std::time::Duration;

/// Where a reading came from, in plain words. `what` is the platform origin a person can go and
/// look at (`/proc/meminfo`, `/sys/class/thermal`, `/`), `detail` is the part that varies per read
/// (which thermal zone won, which battery directory was found).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub what: &'static str,
    pub detail: String,
}

impl Source {
    pub fn new(what: &'static str, detail: impl Into<String>) -> Self {
        Self {
            what,
            detail: detail.into(),
        }
    }

    /// `"/proc/meminfo — MemTotal/MemAvailable"` — the half of §3 rule 1 that is not the number.
    pub fn words(&self) -> String {
        if self.detail.is_empty() {
            self.what.to_string()
        } else {
            format!("{} — {}", self.what, self.detail)
        }
    }

    /// The short form for a hover line: `"read from /proc/meminfo"`.
    pub fn read_from(&self) -> String {
        format!("read from {}", self.what)
    }
}

/// Why there is no reading. One plain sentence via [`Unavailable::human`]; never a code.
#[derive(Clone, Debug, PartialEq)]
pub enum Unavailable {
    /// Not read yet — the cell says `reading…` (design §5.1).
    NotChecked,
    /// The path is not there at all.
    Missing { source: Source },
    /// The read was refused (`EACCES`).
    Denied { source: Source },
    /// There was content and it could not be understood.
    Parse { source: Source, detail: String },
    /// A child failed. `exit` is `None` when it was killed by a signal.
    Command {
        program: &'static str,
        exit: Option<i32>,
        message: String,
    },
    /// A child outlived its budget.
    Timeout {
        program: &'static str,
        after: Duration,
    },
    /// The platform does not expose this at all: `"this firmware does not report fan speed"`.
    NotReported { source: Source, why: &'static str },
    /// A program the app needs is not installed at the path it was resolved to (§5.12).
    ProgramMissing { path: &'static str },
    /// A fake root is in use, so the value would not be a measurement of this machine (§5.13).
    TestMode,
}

impl Unavailable {
    /// One plain sentence, no codes, no paths the reader cannot act on.
    pub fn human(&self) -> String {
        match self {
            Self::NotChecked => "not read yet".to_string(),
            Self::Missing { source } => format!("{} is not there", source.what),
            Self::Denied { source } => format!("permission denied on {}", source.what),
            Self::Parse { source, detail } => format!("{} could not be understood ({detail})", source.what),
            Self::Command {
                program,
                exit,
                message,
            } => match exit {
                Some(code) => format!("{program} exited {code}: {message}"),
                None => format!("{program} was stopped before it answered: {message}"),
            },
            Self::Timeout { program, after } => {
                format!("{program} did not answer within {} s", after.as_secs())
            }
            Self::NotReported { why, .. } => (*why).to_string(),
            Self::ProgramMissing { path } => format!("not installed at {path}"),
            Self::TestMode => "unavailable in test mode".to_string(),
        }
    }

    /// Where the failed read was attempted, when the reason carries one.
    pub fn source(&self) -> Option<&Source> {
        match self {
            Self::Missing { source }
            | Self::Denied { source }
            | Self::Parse { source, .. }
            | Self::NotReported { source, .. } => Some(source),
            Self::Command { .. } | Self::Timeout { .. } | Self::ProgramMissing { .. } => None,
            Self::NotChecked | Self::TestMode => None,
        }
    }
}
