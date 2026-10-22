//! The values the samplers produce and the snapshot carries (design §5.2).
//!
//! Every one of these is plain data read from a documented platform file; the *provenance* lives
//! beside it in a [`Measured`](crate::core::measured::Measured), never inside it.

use std::path::PathBuf;

/// Where the sampler reads from. The defaults are this machine's real roots; a test points them
/// at a fixture tree instead, which is how "an unreadable source renders `unavailable` with a
/// reason" is proved without touching the release binary (design §5.13 is the CLI half).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roots {
    pub proc_root: PathBuf,
    pub sys_root: PathBuf,
    pub etc_root: PathBuf,
    pub root_fs: PathBuf,
}

impl Roots {
    pub fn system() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
            sys_root: PathBuf::from("/sys"),
            etc_root: PathBuf::from("/etc"),
            root_fs: PathBuf::from("/"),
        }
    }
}

/// `/proc/meminfo`, reduced to the two numbers that matter (design §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct MemInfo {
    pub total_kib: u64,
    pub available_kib: u64,
    pub used_pct: f32,
}

impl MemInfo {
    pub fn used_kib(&self) -> u64 {
        self.total_kib.saturating_sub(self.available_kib)
    }

    pub fn used_gib(&self) -> f64 {
        self.used_kib() as f64 / (1024.0 * 1024.0)
    }

    pub fn total_gib(&self) -> f64 {
        self.total_kib as f64 / (1024.0 * 1024.0)
    }
}

/// `statvfs(2)` on the root filesystem — not `df` (design §9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiskInfo {
    pub total_b: u64,
    pub used_b: u64,
    pub free_b: u64,
    pub used_pct: f32,
}

/// The graded value of the root filesystem is how full it is: that is what a health row watches
/// and what an alert's threshold means.
impl From<DiskInfo> for f64 {
    fn from(info: DiskInfo) -> f64 {
        f64::from(info.used_pct)
    }
}

impl DiskInfo {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    pub fn used_gib(&self) -> f64 {
        self.used_b as f64 / Self::GIB
    }

    pub fn total_gib(&self) -> f64 {
        self.total_b as f64 / Self::GIB
    }

    pub fn free_gib(&self) -> f64 {
        self.free_b as f64 / Self::GIB
    }
}

/// What the battery says about itself, plus the firmware's own word for it when that word is not
/// one of the three states we know — `"Not charging"` is reported and is neither of them.
#[derive(Clone, Debug, PartialEq)]
pub struct BatteryInfo {
    pub pct: u32,
    pub state: BatteryState,
    pub raw_status: String,
}

impl BatteryInfo {
    /// `"79 % · charging"`; for a state the firmware names and we do not know, its own word is
    /// quoted rather than guessed at (`"Not charging"` is neither charging nor discharging).
    pub fn words(&self) -> String {
        match self.state {
            BatteryState::Unknown => {
                format!("{} % · the firmware says \"{}\"", self.pct, self.raw_status)
            }
            state => format!("{} % · {}", self.pct, state.word()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    /// The firmware reported a state that is none of the above; `BatteryInfo::raw_status` has
    /// its own word, and the tile prints that word rather than inventing one.
    Unknown,
}

impl BatteryState {
    pub(super) fn parse(word: &str) -> Self {
        match word {
            "Charging" => Self::Charging,
            "Discharging" => Self::Discharging,
            "Full" => Self::Full,
            _ => Self::Unknown,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Charging => "charging",
            Self::Discharging => "discharging",
            Self::Full => "full",
            Self::Unknown => "unknown",
        }
    }
}

/// `/etc/ufw/ufw.conf`, read as two states — an unreadable file is not a third state, it is an
/// [`Unavailable`], so "unknown" can never paint as "open" or as healthy (SEC-V4-26).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirewallState {
    Enabled,
    Disabled,
}

impl FirewallState {
    /// `"enabled"` / `"disabled"` — the word the ribbon prints beside `Firewall`.
    pub fn word(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

impl From<FirewallState> for f64 {
    fn from(state: FirewallState) -> f64 {
        match state {
            FirewallState::Disabled => 0.0,
            FirewallState::Enabled => 1.0,
        }
    }
}
