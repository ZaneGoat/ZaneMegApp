//! The fixture machine: every health test reads a temporary tree, never this laptop's own
//! sensors or battery (design §10.1).

use super::samplers::{now_wall, parse_firewall, parse_meminfo};
use super::subsystems::Grade;
use super::*;
use crate::core::measured::{Belief, Cadence, Effect, Measured, Source, Unavailable, Unit};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

    fn fixture(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zma-health-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("fixture root");
        dir
    }

    fn write(root: &Path, path: &str, contents: &str) {
        let file = root.join(path);
        fs::create_dir_all(file.parent().expect("parent")).expect("dir");
        fs::write(file, contents).expect("file");
    }

    /// A tiny machine: 15.6 GiB of memory 54 % used, one thermal zone at 75 °C, a battery at
    /// 79 %, a firewall, and no fan at all.
    fn fake_machine(root: &Path) {
        write(
            root,
            "proc/meminfo",
            "MemTotal:       16303704 kB\nMemAvailable:    7499704 kB\nSwapTotal:       8388604 kB\n",
        );
        write(root, "sys/class/thermal/thermal_zone0/temp", "75000\n");
        write(root, "sys/class/thermal/thermal_zone1/temp", "51000\n");
        write(root, "sys/class/power_supply/BAT1/capacity", "79\n");
        write(root, "sys/class/power_supply/BAT1/status", "Not charging\n");
        write(root, "etc/ufw/ufw.conf", "# ufw\nENABLED=yes\n");
    }

    fn roots_at(root: &Path) -> Roots {
        Roots {
            proc_root: root.join("proc"),
            sys_root: root.join("sys"),
            etc_root: root.join("etc"),
            root_fs: root.to_path_buf(),
        }
    }

    /// The four tiles' readings come out of the fixture exactly, with the source text attached.
    #[test]
    fn a_fixture_machine_reads_as_its_numbers() {
        let root = fixture("numbers");
        fake_machine(&root);
        let now = Instant::now();
        let sampler = Sampler::new(roots_at(&root), now);
        let snapshot = sampler.snapshot();

        let cpu = snapshot.cpu_temp_c.observed(now).expect("cpu temperature");
        assert_eq!(*cpu.value, 75.0, "the hottest zone wins, not the last one");
        assert!(cpu.source.words().contains("thermal_zone0"), "{:?}", cpu.source);
        assert_eq!(
            snapshot.cpu_temp_c.render(now, Unit::Celsius, 0).as_deref(),
            Some("75 °C")
        );

        let mem = snapshot.mem.observed(now).expect("memory");
        assert_eq!(mem.value.total_kib, 16_303_704);
        assert_eq!((mem.value.used_gib() * 10.0).round() / 10.0, 8.4);
        assert_eq!((mem.value.total_gib() * 10.0).round() / 10.0, 15.5);
        assert!((mem.value.used_pct - 54.0).abs() < 0.1, "{}", mem.value.used_pct);

        let battery = snapshot.battery.observed(now).expect("battery");
        assert_eq!(battery.value.pct, 79);
        assert_eq!(battery.value.state, BatteryState::Unknown, "the firmware says something else");
        assert_eq!(battery.value.raw_status, "Not charging");

        let disk = snapshot.disk_root.observed(now).expect("root filesystem");
        assert!(disk.value.total_b > 0);
        assert!(disk.value.free_b <= disk.value.total_b);

        let firewall = snapshot.firewall.observed(now).expect("firewall");
        assert_eq!(*firewall.value, FirewallState::Enabled);

        let fan = snapshot.fan.reason().expect("a fan reading this machine cannot give");
        assert_eq!(
            fan.human(),
            "this firmware does not report fan speed; the app can only switch Cooler Boost"
        );
        assert_eq!(snapshot.fan.render(now, Unit::Rpm, 0), None);
    }

    /// The other half of AC-11's "or it says unavailable and why": a tree with nothing in it
    /// gives readings that all say `unavailable` with a reason, and no tile formats a number.
    ///
    /// The Cleaner row is exempt because the root filesystem is real: `statvfs` on the fixture
    /// directory answers. The Controller row is exempt for the same reason — the launcher is
    /// looked for under `$HOME`, not under a fake root.
    #[test]
    fn an_unreadable_source_reads_unavailable_with_a_reason() {
        let root = fixture("empty");
        let now = Instant::now();
        let snapshot = Sampler::new(roots_at(&root), now).snapshot().clone();

        for (label, reason) in [
            ("cpu", snapshot.cpu_temp_c.reason().map(Unavailable::human)),
            ("mem", snapshot.mem.reason().map(Unavailable::human)),
            ("battery", snapshot.battery.reason().map(Unavailable::human)),
            ("firewall", snapshot.firewall.reason().map(Unavailable::human)),
            ("fan", snapshot.fan.reason().map(Unavailable::human)),
        ] {
            let reason = reason.unwrap_or_else(|| panic!("{label} claimed a reading on an empty tree"));
            assert!(!reason.is_empty(), "{label} has no sentence");
            assert!(!reason.starts_with('0'), "{label}: {reason}");
        }
        assert_eq!(snapshot.mem.observed(now), None);
        assert_eq!(snapshot.cpu_temp_c.render(now, Unit::Celsius, 0), None);

        let rows = snapshot.health_rows(now);
        for index in [0_usize, 3, 4, 5, 6, 7] {
            let row = &rows[index];
            assert_eq!(
                row.word(),
                HealthWord::Unavailable,
                "{} says {:?} with no reading behind it",
                row.words(),
                row.word()
            );
            assert!(
                row.note().is_some_and(|note| !note.is_empty()),
                "{} has no reason",
                row.words()
            );
        }
    }

    /// AC-14's eight rows, in AC-14's order, and each one says its source in words.
    #[test]
    fn the_health_block_has_one_row_per_subsystem() {
        let root = fixture("rows");
        fake_machine(&root);
        let now = Instant::now();
        let snapshot = Sampler::new(roots_at(&root), now).snapshot().clone();
        let rows = snapshot.health_rows(now);

        let titles: Vec<&str> = rows.iter().map(|row| row.subsystem().title()).collect();
        assert_eq!(
            titles,
            ["Hardware", "Controller", "Cleaner", "Security", "Processes", "Services", "Network", "iDevice"]
        );
        for row in &rows {
            assert!(
                !row.source().is_empty() && row.source() != "no source named",
                "{}",
                row.words()
            );
            assert!(row.words().contains(row.word().word()), "{}", row.words());
        }
        assert_eq!(rows[0].word(), HealthWord::Ok, "75 °C is a good temperature");
        assert_eq!(rows[3].word(), HealthWord::Ok, "an enabled firewall is ok");
        assert_eq!(rows[4].word(), HealthWord::Unavailable, "no process list yet");
        assert_eq!(rows[7].word(), HealthWord::Unavailable, "no device checked yet");
        assert!(rows[4].note().is_some(), "an unavailable row says why");
    }

    /// The word is derived: a stale reading, a failed reading and a live over-threshold reading
    /// each land on their own word, and none of them can be talked into `ok`.
    #[test]
    fn a_word_follows_its_reading() {
        let now = Instant::now();
        let cadence = Cadence::every(Duration::from_secs(1));
        let source = Source::new("/sys/class/thermal", "thermal_zone0");

        let hot = Measured::read(85.0_f32, source.clone(), now, now_wall(), cadence);
        assert_eq!(
            SubsystemHealth::from_measured_graded(Subsystem::Hardware, &hot, now, Grade::AtLeast(80.0)).word(),
            HealthWord::Attention
        );
        let cool = Measured::read(55.0_f32, source.clone(), now, now_wall(), cadence);
        assert_eq!(
            SubsystemHealth::from_measured_graded(Subsystem::Hardware, &cool, now, Grade::AtLeast(80.0)).word(),
            HealthWord::Ok
        );

        let later = now + Duration::from_secs(31);
        let stale = SubsystemHealth::from_measured_graded(Subsystem::Hardware, &cool, later, Grade::AtLeast(80.0));
        assert_eq!(stale.word(), HealthWord::Unavailable, "stale is not live");
        assert!(stale.note().unwrap().starts_with("stale — last read "), "{:?}", stale.note());

        let failed: Measured<f32> = Measured::unavailable(Unavailable::Denied { source });
        let row = SubsystemHealth::from_measured(Subsystem::Hardware, &failed, now);
        assert_eq!(row.word(), HealthWord::Unavailable);
        assert_eq!(row.note(), Some("permission denied on /sys/class/thermal"));

        // `off` only from a live reading that really says off.
        let disabled = Measured::read(FirewallState::Disabled, Source::new("/etc/ufw/ufw.conf", ""), now, now_wall(), cadence);
        assert_eq!(
            SubsystemHealth::from_measured_graded(Subsystem::Security, &disabled, now, Grade::Switch).word(),
            HealthWord::Off
        );
        let unknown: Measured<FirewallState> = Measured::unavailable(Unavailable::TestMode);
        assert_eq!(
            SubsystemHealth::from_measured_graded(Subsystem::Security, &unknown, now, Grade::Switch).word(),
            HealthWord::Unavailable,
            "an unknown firewall is never `off` and never ok"
        );
    }

    /// A requested-only value is never a measurement, in a row or in words (SEC-V4-26).
    #[test]
    fn a_requested_belief_is_never_ok() {
        let now = Instant::now();
        let requested: Belief<u32> = Belief::Requested {
            value: 80,
            at: SystemTime::now(),
            note: "the firmware does not confirm it",
        };
        let row = SubsystemHealth::from_belief(Subsystem::Hardware, &requested, "isw", now);
        assert_eq!(row.word(), HealthWord::Unavailable);
        assert!(row.note().unwrap().contains("not read back"), "{:?}", row.note());

        let confirmed = Effect::Confirmed {
            observed: "Active: active (running)".into(),
        };
        let row = SubsystemHealth::from_effect(Subsystem::Services, &confirmed, "systemctl show");
        assert_eq!(row.word(), HealthWord::Ok);

        let issued = Effect::Requested {
            observed: None,
            note: "cups".into(),
        };
        let row = SubsystemHealth::from_effect(Subsystem::Services, &issued, "systemctl stop");
        assert_eq!(row.word(), HealthWord::Unavailable);
    }

    /// Malformed sources are parsed as `None`, never as a zero.
    #[test]
    fn the_parsers_refuse_what_they_cannot_understand() {
        assert!(parse_meminfo("").is_none());
        assert!(parse_meminfo("MemAvailable: 100 kB\n").is_none());
        assert!(parse_meminfo("MemTotal: not-a-number kB\n").is_none());
        let parsed = parse_meminfo("MemTotal: 1000 kB\nMemAvailable: 250 kB\n").expect("parsed");
        assert_eq!(parsed.used_kib(), 750);
        assert_eq!(parsed.used_pct, 75.0);

        assert_eq!(parse_firewall(""), None);
        assert_eq!(parse_firewall("ENABLED=maybe"), None);
        assert_eq!(parse_firewall("# x\nENABLED=no\n"), Some(FirewallState::Disabled));
        assert_eq!(parse_firewall("ENABLED=yes\n"), Some(FirewallState::Enabled));
        assert_eq!(BatteryState::parse("Charging"), BatteryState::Charging);
        assert_eq!(BatteryState::parse("Discharging"), BatteryState::Discharging);
        assert_eq!(BatteryState::parse("Full"), BatteryState::Full);
        assert_eq!(BatteryState::parse("Not charging"), BatteryState::Unknown);
    }

    /// The slow field is not re-read on a fast tick, and goes stale on its own budget.
    #[test]
    fn each_field_keeps_its_own_cadence_and_age() {
        let root = fixture("cadence");
        fake_machine(&root);
        let start = Instant::now();
        let mut sampler = Sampler::new(roots_at(&root), start);
        let first_disk = sampler.snapshot().disk_root.age(start).expect("disk read");
        assert_eq!(first_disk, Duration::ZERO);

        let later = start + Duration::from_secs(1);
        sampler.tick(later);
        let disk_age = sampler.snapshot().disk_root.age(later).expect("disk still there");
        assert!(
            disk_age >= Duration::from_secs(1),
            "statvfs is a 0.1 Hz read: {disk_age:?}"
        );
        let cpu_age = sampler.snapshot().cpu_temp_c.age(later).expect("cpu");
        assert!(cpu_age < Duration::from_secs(1), "sysfs is a 1 Hz read: {cpu_age:?}");
    }

    /// The battery tile and the ribbon share one sentence, and a state the firmware named and we
    /// do not know is quoted rather than guessed at.
    #[test]
    fn a_battery_word_quotes_an_unknown_state() {
        let unknown = BatteryInfo {
            pct: 79,
            state: BatteryState::Unknown,
            raw_status: "Not charging".to_string(),
        };
        assert_eq!(unknown.words(), "79 % · the firmware says \"Not charging\"");
        let charging = BatteryInfo {
            pct: 79,
            state: BatteryState::Charging,
            raw_status: "Charging".to_string(),
        };
        assert_eq!(charging.words(), "79 % · charging");
    }
