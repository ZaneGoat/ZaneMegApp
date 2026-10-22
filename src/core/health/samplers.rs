//! The readers: `sysfs`, `/proc`, one `statvfs(2)` and two files, and nothing that spawns a
//! process (design §5.2, §9; D19).
//!
//! Each field keeps its own cadence and its own read time, so a field that stops being refreshed
//! goes stale by itself instead of being re-dated by its neighbours.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use super::{BatteryInfo, BatteryState, DiskInfo, FirewallState, MemInfo, Roots};
use super::subsystems::HealthSnapshot;
use crate::core::controller::ControllerManager;
use crate::core::measured::{Cadence, Measured, Source, Unavailable};

/// `sysfs`, `/proc/meminfo`, the battery and the firewall are cheap enough for every tick (the
/// shell ticks once a second); `statvfs` is the 0.1 Hz read of design §9.
const FAST_CADENCE: Duration = Duration::from_secs(1);
const SLOW_CADENCE: Duration = Duration::from_secs(10);

/// The sampler. Every field is refreshed at its own cadence and keeps its own read time, so a
/// field that stops being refreshed goes stale by itself instead of being re-dated by its
/// neighbours.
pub struct Sampler {
    roots: Roots,
    snapshot: HealthSnapshot,
}

impl Sampler {
    /// The real machine.
    pub fn system(now: Instant) -> Self {
        Self::new(Roots::system(), now)
    }

    /// Samples everything once, so the first frame has numbers and not six `reading…` cells
    /// (AC-16). No subprocess runs, and the whole pass is a handful of small file reads.
    pub fn new(roots: Roots, now: Instant) -> Self {
        let mut sampler = Self {
            roots,
            snapshot: HealthSnapshot {
                taken_at: now,
                taken_wall: now_wall(),
                cpu_temp_c: Measured::unavailable(Unavailable::NotChecked),
                mem: Measured::unavailable(Unavailable::NotChecked),
                disk_root: Measured::unavailable(Unavailable::NotChecked),
                battery: Measured::unavailable(Unavailable::NotChecked),
                fan: Measured::unavailable(Unavailable::NotChecked),
                firewall: Measured::unavailable(Unavailable::NotChecked),
                controller_installed: Measured::unavailable(Unavailable::NotChecked),
            },
        };
        sampler.refresh(now, true);
        sampler
    }

    /// Refresh whatever is due. Cheap to call every frame; nothing is read before its cadence
    /// has elapsed.
    pub fn tick(&mut self, now: Instant) {
        self.refresh(now, false);
    }

    pub fn snapshot(&self) -> &HealthSnapshot {
        &self.snapshot
    }

    fn refresh(&mut self, now: Instant, all: bool) {
        let fast = FAST_CADENCE;
        let slow = SLOW_CADENCE;

        if all || due(self.snapshot.cpu_temp_c.age(now), fast) {
            self.snapshot.cpu_temp_c = sample_cpu_temp(&self.roots, now);
        }
        if all || due(self.snapshot.mem.age(now), fast) {
            self.snapshot.mem = sample_mem(&self.roots, now);
        }
        if all || due(self.snapshot.battery.age(now), fast) {
            self.snapshot.battery = sample_battery(&self.roots, now);
        }
        if all || due(self.snapshot.fan.age(now), fast) {
            self.snapshot.fan = sample_fan(&self.roots, now);
        }
        if all || due(self.snapshot.firewall.age(now), fast) {
            self.snapshot.firewall = sample_firewall(&self.roots, now);
        }
        if all || due(self.snapshot.controller_installed.age(now), fast) {
            self.snapshot.controller_installed = sample_controller(now);
        }
        if all || due(self.snapshot.disk_root.age(now), slow) {
            self.snapshot.disk_root = sample_disk(&self.roots, now);
        }
        self.snapshot.taken_at = now;
        self.snapshot.taken_wall = now_wall();
    }
}

fn due(age: Option<Duration>, cadence: Duration) -> bool {
    age.is_none_or(|age| age >= cadence)
}

pub(super) fn now_wall() -> SystemTime {
    SystemTime::now()
}

fn cadence_fast() -> Cadence {
    Cadence::every(FAST_CADENCE)
}

fn cadence_slow() -> Cadence {
    Cadence::every(SLOW_CADENCE)
}

/// `io::Error` → the reason the cell will print. A missing path and a refused one are different
/// sentences because they mean different things to her.
fn unavailable_for(source: &Source, error: &io::Error) -> Unavailable {
    match error.kind() {
        io::ErrorKind::PermissionDenied => Unavailable::Denied {
            source: source.clone(),
        },
        io::ErrorKind::NotFound => Unavailable::Missing {
            source: source.clone(),
        },
        _ => Unavailable::Parse {
            source: source.clone(),
            detail: error.to_string(),
        },
    }
}

fn read_text(path: &Path, source: &Source) -> Result<String, Unavailable> {
    fs::read_to_string(path).map_err(|error| unavailable_for(source, &error))
}

/// The hottest `thermal_zone*/temp` — AC-11 compares the tile against exactly this maximum.
fn sample_cpu_temp(roots: &Roots, now: Instant) -> Measured<f32> {
    let what = "/sys/class/thermal";
    let source = Source::new(what, "max of thermal_zone*/temp");
    let dir = roots.sys_root.join("class").join("thermal");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) => return Measured::unavailable(unavailable_for(&source, &error)),
    };

    let mut hottest: Option<(f32, PathBuf)> = None;
    let mut zones = 0_usize;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let temp_file = path.join("temp");
        let Ok(text) = fs::read_to_string(&temp_file) else {
            continue;
        };
        let Ok(millidegrees) = text.trim().parse::<f32>() else {
            continue;
        };
        zones += 1;
        let celsius = millidegrees / 1000.0;
        if hottest.as_ref().is_none_or(|(best, _)| celsius > *best) {
            hottest = Some((celsius, temp_file));
        }
    }

    match hottest {
        Some((celsius, from)) => Measured::read(
            celsius,
            Source::new(what, from.display().to_string()),
            now,
            now_wall(),
            cadence_fast(),
        ),
        None if zones == 0 => Measured::unavailable(Unavailable::NotReported {
            source,
            why: "this machine exposes no readable thermal zone",
        }),
        None => Measured::unavailable(Unavailable::NotReported {
            source,
            why: "no thermal zone could be read",
        }),
    }
}

/// `/proc/meminfo` → total, available and the percentage used. `used` is `total − available`,
/// which is what `free` reports and what AC-11 compares against.
fn sample_mem(roots: &Roots, now: Instant) -> Measured<MemInfo> {
    let source = Source::new("/proc/meminfo", "MemTotal, MemAvailable");
    let path = roots.proc_root.join("meminfo");
    let text = match read_text(&path, &source) {
        Ok(text) => text,
        Err(reason) => return Measured::unavailable(reason),
    };
    match parse_meminfo(&text) {
        Some(info) => Measured::read(info, source, now, now_wall(), cadence_fast()),
        None => Measured::unavailable(Unavailable::Parse {
            source,
            detail: "no MemTotal line".to_string(),
        }),
    }
}

/// The pure half of [`sample_mem`], so a fixture is a string and not a machine.
pub(super) fn parse_meminfo(text: &str) -> Option<MemInfo> {
    let mut total_kib = 0_u64;
    let mut available_kib = 0_u64;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        match key {
            "MemTotal:" => total_kib = value,
            "MemAvailable:" => available_kib = value,
            _ => {}
        }
    }
    if total_kib == 0 {
        return None;
    }
    let used_kib = total_kib.saturating_sub(available_kib);
    Some(MemInfo {
        total_kib,
        available_kib,
        used_pct: (used_kib as f32 / total_kib as f32) * 100.0,
    })
}

/// `statvfs(2)` on the root filesystem: total, used and available bytes.
fn sample_disk(roots: &Roots, now: Instant) -> Measured<DiskInfo> {
    let source = Source::new(
        "/",
        format!("statvfs(2) on {}", roots.root_fs.display()),
    );
    match statvfs(&roots.root_fs) {
        Ok(info) => Measured::read(info, source, now, now_wall(), cadence_slow()),
        Err(reason) => Measured::unavailable(reason),
    }
}

fn statvfs(path: &Path) -> Result<DiskInfo, Unavailable> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let source = Source::new("/", format!("statvfs(2) on {}", path.display()));
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| Unavailable::Parse {
        source: source.clone(),
        detail: "the path contains a NUL byte".to_string(),
    })?;
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a valid NUL-terminated string and `stats` is a zeroed, correctly
    // sized `libc::statvfs` owned by this function.
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stats) };
    if rc != 0 {
        return Err(unavailable_for(&source, &io::Error::last_os_error()));
    }
    let block = stats.f_frsize;
    let total_b = stats.f_blocks.saturating_mul(block);
    let used_b = stats.f_blocks.saturating_sub(stats.f_bfree).saturating_mul(block);
    let free_b = stats.f_bavail.saturating_mul(block);
    let denominator = used_b.saturating_add(free_b);
    let used_pct = if denominator == 0 {
        0.0
    } else {
        (used_b as f32 / denominator as f32) * 100.0
    };
    Ok(DiskInfo {
        total_b,
        used_b,
        free_b,
        used_pct,
    })
}

/// The first `BAT*` under `power_supply`, its capacity and the firmware's own status word.
fn sample_battery(roots: &Roots, now: Instant) -> Measured<BatteryInfo> {
    let dir = roots.sys_root.join("class").join("power_supply");
    let source = Source::new("/sys/class/power_supply", "BAT*/capacity, BAT*/status");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) => return Measured::unavailable(unavailable_for(&source, &error)),
    };

    let mut battery_dir: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("BAT") {
            battery_dir = Some(entry.path());
            break;
        }
    }
    let Some(battery_dir) = battery_dir else {
        return Measured::unavailable(Unavailable::NotReported {
            source,
            why: "this machine reports no battery",
        });
    };

    let capacity_file = battery_dir.join("capacity");
    let status_file = battery_dir.join("status");
    let source = Source::new(
        "/sys/class/power_supply",
        format!("{}/capacity, {}/status", battery_dir.display(), battery_dir.display()),
    );
    let capacity_text = match read_text(&capacity_file, &source) {
        Ok(text) => text,
        Err(reason) => return Measured::unavailable(reason),
    };
    let Ok(pct) = capacity_text.trim().parse::<u32>() else {
        return Measured::unavailable(Unavailable::Parse {
            source,
            detail: "capacity was not a number".to_string(),
        });
    };
    let raw_status = fs::read_to_string(&status_file)
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    if raw_status.is_empty() {
        return Measured::unavailable(Unavailable::Parse {
            source,
            detail: "status was empty".to_string(),
        });
    }
    let info = BatteryInfo {
        pct,
        state: BatteryState::parse(&raw_status),
        raw_status,
    };
    Measured::read(info, source, now, now_wall(), cadence_fast())
}

/// Fan speed from `hwmon`. This machine reports none — no `fan*_input` exists under
/// `/sys/class/hwmon` — which is a fact worth saying in words rather than showing as `0 rpm`.
fn sample_fan(roots: &Roots, now: Instant) -> Measured<u32> {
    let hwmon = roots.sys_root.join("class").join("hwmon");
    let mut fastest: Option<(u32, PathBuf)> = None;
    if let Ok(entries) = fs::read_dir(&hwmon) {
        for entry in entries.flatten() {
            let dir = entry.path();
            let Ok(files) = fs::read_dir(&dir) else {
                continue;
            };
            for file in files.flatten() {
                let name = file.file_name().to_string_lossy().to_string();
                if !(name.starts_with("fan") && name.ends_with("_input")) {
                    continue;
                }
                let Ok(text) = fs::read_to_string(file.path()) else {
                    continue;
                };
                let Ok(rpm) = text.trim().parse::<u32>() else {
                    continue;
                };
                if fastest.as_ref().is_none_or(|(best, _)| rpm > *best) {
                    fastest = Some((rpm, file.path()));
                }
            }
        }
    }
    match fastest {
        Some((rpm, from)) => Measured::read(
            rpm,
            Source::new("/sys/class/hwmon", from.display().to_string()),
            now,
            now_wall(),
            cadence_fast(),
        ),
        None => Measured::unavailable(Unavailable::NotReported {
            source: Source::new("/sys/class/hwmon", "no fan*_input on this machine"),
            why: "this firmware does not report fan speed; the app can only switch Cooler Boost",
        }),
    }
}

/// `/etc/ufw/ufw.conf`'s `ENABLED=` line — the world-readable source D19 replaces `sudo -n ufw
/// status` with (no `sudo` on the UI thread, no password prompt out of nowhere).
fn sample_firewall(roots: &Roots, now: Instant) -> Measured<FirewallState> {
    let source = Source::new("/etc/ufw/ufw.conf", "ENABLED=");
    let path = roots.etc_root.join("ufw").join("ufw.conf");
    let text = match read_text(&path, &source) {
        Ok(text) => text,
        Err(reason) => return Measured::unavailable(reason),
    };
    match parse_firewall(&text) {
        Some(state) => Measured::read(state, source, now, now_wall(), cadence_fast()),
        None => Measured::unavailable(Unavailable::Parse {
            source,
            detail: "no ENABLED= line".to_string(),
        }),
    }
}

pub(super) fn parse_firewall(text: &str) -> Option<FirewallState> {
    for line in text.lines() {
        let line = line.trim();
        let Some(value) = line.strip_prefix("ENABLED=") else {
            continue;
        };
        return match value.trim() {
            "yes" => Some(FirewallState::Enabled),
            "no" => Some(FirewallState::Disabled),
            _ => None,
        };
    }
    None
}

/// Whether the Controller Space launcher exists. A missing script is a `Missing` reason, not a
/// live `false`, so "not installed" cannot be confused with "switched off".
fn sample_controller(now: Instant) -> Measured<bool> {
    let manager = ControllerManager::new();
    let source = Source::new(
        "~/Desktop/Controler space/start.sh",
        "the Controller Space launcher",
    );
    if manager.is_installed() {
        Measured::read(true, source, now, now_wall(), cadence_fast())
    } else {
        Measured::unavailable(Unavailable::Missing { source })
    }
}

