//! The Mission Control alert strip, as data (design §5.3, AC-12).
//!
//! A rule is a row in [`RULES`]: which reading it watches, the comparison, the threshold, the
//! sentence she reads, and where it sends her. [`evaluate`] is pure — same snapshot and same
//! `now` in, same alerts out — and it reaches every value through
//! [`Measured::observed`](crate::core::measured::Measured::observed), so a stale or failed
//! reading can never fire an alert: it produces a *health* row instead (SEC-V4-25).
//!
//! Nothing here changes anything. An alert says a true sentence and offers a destination.

use std::time::Instant;

use crate::app::tabs::AppTab;
use crate::core::health::HealthSnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertId {
    CpuHot,
    RamHigh,
    DiskFull,
    BatteryLow,
    FirewallOpen,
    ServicesFailed,
}

/// Which reading in the snapshot a rule watches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    CpuTemp,
    RamPct,
    DiskPct,
    BatteryPct,
    Firewall,
    /// No sampler for this yet (the services list lands with its tab in 4b), so the rule exists,
    /// is documented, and cannot fire — which is the truth, not a disabled button.
    FailedUnits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compare {
    /// Fires at or above the threshold.
    Gte,
    /// Fires at or below the threshold.
    Lte,
    /// Fires when a switch is really off.
    Off,
}

/// Where an alert sends her. `tab` is `None` while the fixing tab does not exist, and the strip
/// then says so instead of offering a control that goes nowhere (AC-13, §3 rule 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Destination {
    pub label: &'static str,
    pub tab: Option<AppTab>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    pub id: AlertId,
    /// The reading's own name on screen: `RAM`, `Root disk`, `CPU`.
    pub label: &'static str,
    /// What it means, in one plain sentence.
    pub sentence: &'static str,
    pub reading: Reading,
    pub compare: Compare,
    pub threshold: f64,
    pub destination: Destination,
}

impl Rule {
    /// `"RAM 91 % (14.2 of 15.6 GiB) → Processes"` — the measured value with its unit, and where
    /// to go (design §5.3).
    pub fn headline(&self, measured: &str) -> String {
        format!("{} {} → {}", self.label, measured, self.destination.label)
    }
}

const fn tab(label: &'static str, tab: AppTab) -> Destination {
    Destination {
        label,
        tab: Some(tab),
    }
}

/// AC-12's six rules, in the order they are read out. The thresholds are the contract's.
pub const RULES: [Rule; 6] = [
    Rule {
        id: AlertId::CpuHot,
        label: "CPU",
        sentence: "the processor is running hot",
        reading: Reading::CpuTemp,
        compare: Compare::Gte,
        threshold: 80.0,
        destination: tab("Hardware", AppTab::Hardware),
    },
    Rule {
        id: AlertId::RamHigh,
        label: "RAM",
        sentence: "memory is nearly full",
        reading: Reading::RamPct,
        compare: Compare::Gte,
        threshold: 85.0,
        destination: Destination {
            label: "Processes",
            tab: None,
        },
    },
    Rule {
        id: AlertId::DiskFull,
        label: "Root disk",
        sentence: "the root filesystem is nearly full",
        reading: Reading::DiskPct,
        compare: Compare::Gte,
        threshold: 90.0,
        destination: tab("System Cleaner", AppTab::Cleaner),
    },
    Rule {
        id: AlertId::BatteryLow,
        label: "Battery",
        sentence: "the battery is low and the laptop is running on it",
        reading: Reading::BatteryPct,
        compare: Compare::Lte,
        threshold: 20.0,
        destination: tab("Hardware", AppTab::Hardware),
    },
    Rule {
        id: AlertId::FirewallOpen,
        label: "Firewall",
        sentence: "incoming connections are allowed",
        reading: Reading::Firewall,
        compare: Compare::Off,
        threshold: 0.0,
        destination: tab("Network & Guard", AppTab::Security),
    },
    Rule {
        id: AlertId::ServicesFailed,
        label: "Services",
        sentence: "a service has failed",
        reading: Reading::FailedUnits,
        compare: Compare::Gte,
        threshold: 0.5,
        destination: Destination {
            label: "Services",
            tab: None,
        },
    },
];

/// One firing rule, with the value that made it fire written out.
#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub rule: &'static Rule,
    /// The value words, with the unit: `"91 % (14.2 of 15.6 GiB)"`, `"79 % · charging"`.
    pub measured: String,
    /// `"RAM 91 % (14.2 of 15.6 GiB) → Processes"`.
    pub headline: String,
}

/// Pure. Same snapshot and same `now` in → the same alerts out. An empty result renders
/// `All systems nominal` and nothing is alert-coloured.
pub fn evaluate(snapshot: &HealthSnapshot, now: Instant) -> Vec<Alert> {
    let mut alerts = Vec::new();
    for rule in &RULES {
        let Some(measured) = fired(rule, snapshot, now) else {
            continue;
        };
        alerts.push(Alert {
            rule,
            headline: rule.headline(&measured),
            measured,
        });
    }
    alerts
}

/// The value words when the rule fires, `None` when it does not — or when there is no live
/// reading to judge, which is never an alert (design §5.3).
fn fired(rule: &Rule, snapshot: &HealthSnapshot, now: Instant) -> Option<String> {
    match rule.reading {
        Reading::CpuTemp => {
            let observed = snapshot.cpu_temp_c.observed(now)?;
            let celsius = f64::from(*observed.value);
            matches(rule.compare, celsius, rule.threshold)
                .then(|| format!("{celsius:.0} °C"))
        }
        Reading::RamPct => {
            let observed = snapshot.mem.observed(now)?;
            let mem = observed.value;
            let percent = f64::from(mem.used_pct);
            matches(rule.compare, percent, rule.threshold).then(|| {
                format!(
                    "{percent:.0} % ({:.1} of {:.1} GiB)",
                    mem.used_gib(),
                    mem.total_gib()
                )
            })
        }
        Reading::DiskPct => {
            let observed = snapshot.disk_root.observed(now)?;
            let disk = observed.value;
            let percent = f64::from(disk.used_pct);
            matches(rule.compare, percent, rule.threshold).then(|| {
                format!(
                    "{percent:.0} % ({:.1} of {:.1} GiB used, {:.1} GiB free)",
                    disk.used_gib(),
                    disk.total_gib(),
                    disk.free_gib()
                )
            })
        }
        Reading::BatteryPct => {
            let observed = snapshot.battery.observed(now)?;
            let battery = observed.value;
            let percent = f64::from(battery.pct);
            // "discharging" is part of the rule, not decoration: a full battery on the charger
            // is not a low battery. A state the firmware did not name is not "discharging".
            let discharging = battery.state == crate::core::health::BatteryState::Discharging;
            (discharging && matches(rule.compare, percent, rule.threshold))
                .then(|| format!("{percent:.0} % · discharging"))
        }
        Reading::Firewall => {
            let observed = snapshot.firewall.observed(now)?;
            let enabled = f64::from(*observed.value);
            match rule.compare {
                Compare::Off => (enabled < 0.5).then(|| "open · /etc/ufw/ufw.conf".to_string()),
                other => matches(other, enabled, rule.threshold).then(|| "on".to_string()),
            }
        }
        // Nothing samples this yet, so nothing can be true about it (4b owns the services list).
        Reading::FailedUnits => None,
    }
}

fn matches(compare: Compare, value: f64, threshold: f64) -> bool {
    match compare {
        Compare::Gte => value >= threshold,
        Compare::Lte => value <= threshold,
        Compare::Off => value < 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::health::{BatteryInfo, BatteryState, DiskInfo, FirewallState, MemInfo};
    use crate::core::measured::{Cadence, Measured, Source, Unavailable};
    use std::time::{Duration, SystemTime};

    const CADENCE: Duration = Duration::from_secs(1);

    fn snapshot(now: Instant) -> HealthSnapshot {
        let source = |what: &'static str| Source::new(what, "");
        HealthSnapshot {
            taken_at: now,
            taken_wall: SystemTime::now(),
            cpu_temp_c: Measured::read(55.0_f32, source("/sys/class/thermal"), now, SystemTime::now(), Cadence::every(CADENCE)),
            mem: Measured::read(
                MemInfo {
                    total_kib: 16_303_704,
                    available_kib: 8_804_000,
                    used_pct: 54.0,
                },
                source("/proc/meminfo"),
                now,
                SystemTime::now(),
                Cadence::every(CADENCE),
            ),
            disk_root: Measured::read(
                DiskInfo {
                    total_b: 80_000_000_000,
                    used_b: 20_000_000_000,
                    free_b: 60_000_000_000,
                    used_pct: 25.0,
                },
                source("/"),
                now,
                SystemTime::now(),
                Cadence::every(CADENCE),
            ),
            battery: Measured::read(
                BatteryInfo {
                    pct: 79,
                    state: BatteryState::Discharging,
                    raw_status: "Discharging".into(),
                },
                source("/sys/class/power_supply/BAT1"),
                now,
                SystemTime::now(),
                Cadence::every(CADENCE),
            ),
            fan: Measured::unavailable(Unavailable::NotReported {
                source: source("/sys/class/hwmon"),
                why: "this firmware does not report fan speed",
            }),
            firewall: Measured::read(FirewallState::Enabled, source("/etc/ufw/ufw.conf"), now, SystemTime::now(), Cadence::every(CADENCE)),
            controller_installed: Measured::unavailable(Unavailable::NotChecked),
        }
    }

    /// Nothing is wrong with this machine, so the strip says exactly that (AC-12).
    #[test]
    fn a_healthy_machine_produces_no_alerts() {
        let now = Instant::now();
        let alerts = evaluate(&snapshot(now), now);
        assert!(alerts.is_empty(), "{alerts:?}");
    }

    /// Each rule fires on its own value, quotes it with the unit, and names its destination.
    #[test]
    fn every_rule_fires_on_its_own_value() {
        let now = Instant::now();

        let mut hot = snapshot(now);
        hot.cpu_temp_c = Measured::read(85.0, Source::new("/sys/class/thermal", ""), now, SystemTime::now(), Cadence::every(CADENCE));
        let alerts = evaluate(&hot, now);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].rule.id, AlertId::CpuHot);
        assert_eq!(alerts[0].measured, "85 °C");
        assert_eq!(alerts[0].headline, "CPU 85 °C → Hardware");
        assert_eq!(alerts[0].rule.destination.tab, Some(AppTab::Hardware));

        let mut full = snapshot(now);
        full.mem = Measured::read(
            MemInfo {
                total_kib: 16_303_704,
                available_kib: 1_467_000,
                used_pct: 91.0,
            },
            Source::new("/proc/meminfo", ""),
            now,
            SystemTime::now(),
            Cadence::every(CADENCE),
        );
        let alerts = evaluate(&full, now);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].rule.id, AlertId::RamHigh);
        assert!(alerts[0].measured.starts_with("91 % ("), "{}", alerts[0].measured);
        assert!(alerts[0].measured.contains("GiB"), "{}", alerts[0].measured);
        // The Processes tab does not exist yet: the rule still fires, and says it has no door.
        assert_eq!(alerts[0].rule.destination.tab, None);
        assert_eq!(alerts[0].headline, format!("RAM {} → Processes", alerts[0].measured));

        let mut disk = snapshot(now);
        disk.disk_root = Measured::read(
            DiskInfo {
                total_b: 80_000_000_000,
                used_b: 76_000_000_000,
                free_b: 4_000_000_000,
                used_pct: 95.0,
            },
            Source::new("/", ""),
            now,
            SystemTime::now(),
            Cadence::every(CADENCE),
        );
        let alerts = evaluate(&disk, now);
        assert_eq!(alerts[0].rule.id, AlertId::DiskFull);
        assert!(alerts[0].measured.contains("95 %"), "{}", alerts[0].measured);

        let mut low = snapshot(now);
        low.battery = Measured::read(
            BatteryInfo {
                pct: 15,
                state: BatteryState::Discharging,
                raw_status: "Discharging".into(),
            },
            Source::new("/sys/class/power_supply/BAT1", ""),
            now,
            SystemTime::now(),
            Cadence::every(CADENCE),
        );
        let alerts = evaluate(&low, now);
        assert_eq!(alerts[0].rule.id, AlertId::BatteryLow);
        assert_eq!(alerts[0].measured, "15 % · discharging");

        let mut open = snapshot(now);
        open.firewall = Measured::read(FirewallState::Disabled, Source::new("/etc/ufw/ufw.conf", ""), now, SystemTime::now(), Cadence::every(CADENCE));
        let alerts = evaluate(&open, now);
        assert_eq!(alerts[0].rule.id, AlertId::FirewallOpen);
        assert_eq!(alerts[0].rule.destination.tab, Some(AppTab::Security));
    }

    /// The battery rule is about a battery that is being drained, not about a low number while
    /// the laptop sits on the charger.
    #[test]
    fn the_battery_rule_needs_a_discharging_battery() {
        let now = Instant::now();
        for state in [BatteryState::Charging, BatteryState::Full, BatteryState::Unknown] {
            let mut snapshot = snapshot(now);
            snapshot.battery = Measured::read(
                BatteryInfo {
                    pct: 10,
                    state,
                    raw_status: format!("{state:?}"),
                },
                Source::new("/sys/class/power_supply/BAT1", ""),
                now,
                SystemTime::now(),
                Cadence::every(CADENCE),
            );
            assert!(evaluate(&snapshot, now).is_empty(), "{state:?} fired the low-battery rule");
        }
    }

    /// A stale or failed reading is never an alert: it is a health row (SEC-V4-25, T-31).
    #[test]
    fn nothing_fires_on_a_stale_or_missing_reading() {
        let read_at = Instant::now();
        let mut snapshot = snapshot(read_at);
        snapshot.cpu_temp_c = Measured::read(95.0, Source::new("/sys/class/thermal", ""), read_at, SystemTime::now(), Cadence::every(CADENCE));
        snapshot.mem = Measured::unavailable(Unavailable::Denied {
            source: Source::new("/proc/meminfo", ""),
        });
        snapshot.disk_root = Measured::unavailable(Unavailable::TestMode);
        snapshot.firewall = Measured::unavailable(Unavailable::Missing {
            source: Source::new("/etc/ufw/ufw.conf", ""),
        });

        let just_inside = read_at + Duration::from_secs(3);
        assert_eq!(evaluate(&snapshot, just_inside).len(), 1, "the cpu reading is still live");

        let later = read_at + Duration::from_secs(4);
        assert!(
            evaluate(&snapshot, later).is_empty(),
            "nothing may fire on a stale reading"
        );
    }

    /// The services rule is present and cannot fire, because nothing samples failed units yet.
    #[test]
    fn the_services_rule_is_documented_and_inert() {
        let now = Instant::now();
        let rule = RULES
            .iter()
            .find(|rule| rule.id == AlertId::ServicesFailed)
            .expect("the failed-units rule is in the table");
        assert_eq!(rule.reading, Reading::FailedUnits);
        assert!(fired(rule, &snapshot(now), now).is_none());
    }
}
