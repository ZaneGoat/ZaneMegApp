use super::*;
use std::time::Duration;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zma-log-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn iso8601_is_utc_and_handles_leap_days() {
    assert_eq!(format_iso8601(0), "1970-01-01T00:00:00Z");
    assert_eq!(format_iso8601(1_700_000_000), "2023-11-14T22:13:20Z");
    // 2024-02-29 is a leap day: the day after 2024-02-28.
    assert_eq!(format_iso8601(1_709_164_800), "2024-02-29T00:00:00Z");
    assert_eq!(format_iso8601(1_709_251_199), "2024-02-29T23:59:59Z");
    let now = iso8601_utc(SystemTime::UNIX_EPOCH + Duration::from_secs(86_399));
    assert_eq!(now, "1970-01-01T23:59:59Z");
}

#[test]
fn redaction_removes_home_hostname_mac_and_ips() {
    let home = home_dir()
        .map(|home| home.display().to_string())
        .unwrap_or_default();
    let text = format!(
        "opened {home}/secrets.txt from 192.168.1.24 aa:bb:cc:dd:ee:ff 2001:db8::1 host=localhost"
    );
    let redacted = redact_and_escape(&text);
    if !home.is_empty() && home.len() > 1 {
        assert!(!redacted.contains(&home), "home path survived: {redacted}");
    }
    assert!(
        !redacted.contains("192.168.1.24"),
        "IPv4 survived: {redacted}"
    );
    assert!(
        !redacted.contains("aa:bb:cc:dd:ee:ff"),
        "MAC survived: {redacted}"
    );
    assert!(
        !redacted.contains("2001:db8::1"),
        "IPv6 survived: {redacted}"
    );
    assert!(
        redacted.contains("<ip>") && redacted.contains("<mac>"),
        "no markers: {redacted}"
    );
}

#[test]
fn redaction_keeps_symbols_and_timestamps_readable() {
    let text = "frame=zanemegapp::ui::tab_hardware::TabHardware::ui at 08:00:00 e5::a1";
    let redacted = redact_and_escape(text);
    assert_eq!(
        redacted, text,
        "a symbol path must not be redacted as an IP"
    );
}

#[test]
fn a_newline_cannot_forge_a_second_event() {
    let detail = "bad\n2026-10-20T09:00:00Z PANIC version=\"forged\"";
    let escaped = redact_and_escape(detail);
    assert!(!escaped.contains('\n'));
    assert_eq!(escaped.matches("PANIC").count(), 1);
    assert!(escaped.contains("\\n2026-10-20T09:00:00Z"));
}

#[test]
fn caps_a_single_line_at_four_kibibytes() {
    let long = "x".repeat(MAX_LINE_BYTES * 2);
    let capped = cap_line(&long);
    assert_eq!(capped.len(), MAX_LINE_BYTES);
    assert!(capped.ends_with('\n'));
    let short = "ok\n";
    assert_eq!(cap_line(short).as_ref(), short);
}

#[test]
fn writer_creates_a_0600_file_in_a_0700_dir_and_appends() {
    let dir = temp_dir("append");
    let mut logger = Logger::open(&dir);
    assert_eq!(
        logger.degraded(),
        None,
        "logger degraded: {:?}",
        logger.degraded()
    );
    logger.write(Level::Info, "startup", "version=\"ZaneMegApp 4.0.0\"");
    logger.write(Level::Error, "action.failed", "reason=\"Access denied\"");

    let path = logger.path().expect("log path");
    let first = std::fs::read_to_string(&path).expect("log file");
    assert!(
        first.contains("event=startup"),
        "missing startup line: {first}"
    );
    assert!(
        first.contains("ZaneMegApp 4.0.0"),
        "version missing: {first}"
    );
    assert!(first.contains("event=action.failed"));
    assert_eq!(first.lines().count(), 2, "one line per event: {first}");

    // Re-opening appends; it must never truncate what is already there.
    drop(logger);
    let mut again = Logger::open(&dir);
    again.write(Level::Info, "again", "");
    let second = std::fs::read_to_string(&path).expect("log file");
    assert!(
        second.starts_with(&first),
        "an existing log was truncated: {second}"
    );

    use std::os::unix::fs::PermissionsExt as _;
    let file_mode = std::fs::metadata(&path)
        .expect("log metadata")
        .permissions()
        .mode()
        & 0o777;
    let dir_mode = std::fs::metadata(&dir)
        .expect("dir metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file_mode, 0o600, "log file mode");
    assert_eq!(dir_mode, 0o700, "log dir mode");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_planted_symlink_is_never_followed() {
    let dir = temp_dir("symlink");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let victim = dir.join("victim.txt");
    std::fs::write(&victim, b"untouched").expect("victim");
    std::os::unix::fs::symlink(&victim, dir.join(FILE_NAME)).expect("planted symlink");

    let mut logger = Logger::open(&dir);
    assert!(logger.degraded().is_some(), "a symlinked log must degrade");
    logger.write(Level::Info, "startup", "");
    assert_eq!(std::fs::read(&victim).expect("victim"), b"untouched");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_append_path_is_bounded() {
    let dir = temp_dir("bounded");
    let mut logger = Logger::open(&dir);
    let detail = "y".repeat(MAX_LINE_BYTES * 2);
    for _ in 0..8 {
        logger.write(Level::Info, "soak", &detail);
    }
    let path = logger.path().expect("log path");
    let len = std::fs::metadata(&path).expect("metadata").len();
    assert!(len <= MAX_BYTES, "log grew past the cap: {len}");
    let content = std::fs::read_to_string(&path).expect("log");
    for line in content.lines() {
        assert!(
            line.len() <= MAX_LINE_BYTES,
            "line over the cap: {}",
            line.len()
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_panic_line_carries_version_message_and_symbol_without_a_path() {
    let dir = temp_dir("panic");
    let mut logger = Logger::open(&dir);
    logger.write_panic(
        "Negative width makes no sense, but got: -6",
        Some("zanemegapp::ui::tab_hardware::TabHardware::ui"),
    );
    let path = logger.path().expect("log path");
    let line = std::fs::read_to_string(&path).expect("log");
    assert_eq!(line.lines().count(), 1, "a panic is one line: {line}");
    assert!(line.contains("PANIC"));
    assert!(line.contains("ZaneMegApp 4.0.0"));
    assert!(line.contains("Negative width makes no sense"));
    assert!(line.contains("zanemegapp::ui::tab_hardware::TabHardware::ui"));
    assert!(
        !line.contains(".rs:"),
        "no file or line numbers in the log: {line}"
    );
    assert!(!line.contains("src/"), "no source path in the log: {line}");
    let home = home_dir()
        .map(|home| home.display().to_string())
        .unwrap_or_default();
    if home.len() > 1 {
        assert!(!line.contains(&home), "home path in the log: {line}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
