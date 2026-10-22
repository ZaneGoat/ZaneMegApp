//! The app's log writer and panic hook — AC-03, AC-47, AC-48, SEC-V4-19…23.
//!
//! One line per event, ISO-8601 UTC, written to `$XDG_CONFIG_HOME/zanemegapp/logs/zanemegapp.log`
//! (default `~/.config/zanemegapp/logs/`; design Q3/§5.5, spec AC-48). The directory is created
//! `0700` and verified through a held fd (`O_DIRECTORY|O_NOFOLLOW`, `fstat` uid + mode), the file
//! is opened `O_WRONLY|O_CREAT|O_APPEND|O_NOFOLLOW` `0600` and never truncated (SEC-V4-21). Any
//! failure degrades to stderr — it never fails the launch — and says so on the first line.
//!
//! Everything a caller passes goes through [`redact_and_escape`] *inside* the writer, so a new
//! call site cannot forget it (SEC-V4-22): `$HOME` becomes `~`, the hostname `<host>`, MACs
//! `<mac>`, IP literals `<ip>`, and quotes/newlines are escaped so one event is one line.
//!
//! Rotation at 1 MiB is a TODO for slice 4c (`--self-test-log-fill` proves it); the append path
//! itself is already bounded: a single line is capped at [`MAX_LINE_BYTES`] and once the file
//! would pass [`MAX_BYTES`] the writer stops instead of growing without limit.

use std::ffi::CString;
use std::fs::File;
use std::io::Write as _;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::os::unix::io::{AsRawFd, FromRawFd as _, RawFd};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single log line never exceeds this many bytes (SEC-V4-21).
pub const MAX_LINE_BYTES: usize = 4096;
/// The log rotates at this size (AC-48); 4c owns the rotation path.
pub const MAX_BYTES: u64 = 1024 * 1024;
pub const FILE_NAME: &str = "zanemegapp.log";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Info,
    /// Reserved for slice 4c's first warning call site (config recovery, log degraded).
    #[allow(dead_code)]
    Warn,
    /// Reserved for slice 4b/4c's failure call sites (action failed, worker fault).
    #[allow(dead_code)]
    Error,
}

impl Level {
    pub fn word(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// The one writer. `file` is `None` when the directory or the file could not be verified,
/// in which case every line goes to stderr and [`Logger::degraded`] names the reason.
pub struct Logger {
    dir: PathBuf,
    file: Option<File>,
    bytes: u64,
    degraded: Option<String>,
}

impl Logger {
    /// Never fails: an unusable directory yields a stderr-only logger with a reason.
    pub fn open(dir: &Path) -> Self {
        let mut logger = Self {
            dir: dir.to_path_buf(),
            file: None,
            bytes: 0,
            degraded: None,
        };
        match open_verified_dir(dir) {
            Ok(dirfd) => match open_log_file(dirfd.as_raw_fd()) {
                Ok((file, len)) => {
                    logger.file = Some(file);
                    logger.bytes = len;
                }
                Err(err) => {
                    logger.degraded = Some(format!("{}: {err}", dir.join(FILE_NAME).display()));
                }
            },
            Err(err) => logger.degraded = Some(format!("{}: {err}", dir.display())),
        }
        logger
    }

    /// `Some(path)` when lines are really going to a file.
    pub fn path(&self) -> Option<PathBuf> {
        self.file.as_ref().map(|_| self.dir.join(FILE_NAME))
    }

    /// The reason logging is on stderr instead of disk, if it is.
    pub fn degraded(&self) -> Option<&str> {
        self.degraded.as_deref()
    }

    /// The fd the panic hook falls back to when the mutex is held or poisoned (SEC-V4-23).
    pub fn raw_fd(&self) -> Option<RawFd> {
        self.file.as_ref().map(AsRawFd::as_raw_fd)
    }

    /// One event, one line. Both fields are redacted and escaped here, not by the caller.
    pub fn write(&mut self, level: Level, event: &str, detail: &str) {
        let line = format!(
            "{} {:<5} event={} {}\n",
            iso8601_utc(SystemTime::now()),
            level.word(),
            redact_and_escape(event),
            redact_and_escape(detail),
        );
        self.emit(&line);
    }

    /// The crash line: `PANIC` plus message and symbol, no path, no environment (AC-03/AC-47).
    pub fn write_panic(&mut self, message: &str, frame: Option<&str>) {
        let line = panic_line(message, frame);
        self.emit(&line);
    }

    fn emit(&mut self, line: &str) {
        let line = cap_line(line);
        let Some(file) = self.file.as_mut() else {
            let _ = std::io::stderr().write_all(line.as_bytes());
            return;
        };
        if self.bytes + line.len() as u64 > MAX_BYTES {
            self.degraded = Some(format!(
                "{} is full ({MAX_BYTES} bytes) and rotation lands in slice 4c — this line went to stderr",
                self.dir.join(FILE_NAME).display()
            ));
            let _ = std::io::stderr().write_all(line.as_bytes());
            return;
        }
        if file.write_all(line.as_bytes()).is_ok() {
            self.bytes += line.len() as u64;
        } else {
            self.degraded = Some(format!("write to {} failed", self.dir.display()));
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
    }
}

/// The crash line as text — one line, used by the writer, the raw-fd fallback and stderr.
pub fn panic_line(message: &str, frame: Option<&str>) -> String {
    let line = format!(
        "{} {:<5} version=\"{}\" message=\"{}\" frame=\"{}\"\n",
        iso8601_utc(SystemTime::now()),
        "PANIC",
        crate::app_version(),
        redact_and_escape(message),
        redact_and_escape(frame.unwrap_or("<unknown>")),
    );
    cap_line(&line).into_owned()
}

/// Truncates at the last whole character boundary under [`MAX_LINE_BYTES`] and keeps the newline.
pub fn cap_line(line: &str) -> std::borrow::Cow<'_, str> {
    if line.len() <= MAX_LINE_BYTES {
        return std::borrow::Cow::Borrowed(line);
    }
    let mut end = MAX_LINE_BYTES - 1;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    let mut capped = String::with_capacity(end + 1);
    capped.push_str(line.get(..end).unwrap_or_default());
    capped.push('\n');
    std::borrow::Cow::Owned(capped)
}

fn open_verified_dir(dir: &Path) -> Result<File, String> {
    use std::os::unix::fs::DirBuilderExt as _;

    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("cannot create {}: {err}", parent.display()))?;
    }
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700);
    match builder.create(dir) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(err) => return Err(format!("cannot create: {err}")),
    }

    let c_path =
        CString::new(dir.as_os_str().as_bytes()).map_err(|_| "path contains NUL".to_string())?;
    // SAFETY: `c_path` is a valid NUL-terminated path; the fd is owned immediately.
    let fd = unsafe {
        libc::open(
            c_path.as_ptr(),
            libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_RDONLY,
        )
    };
    if fd < 0 {
        return Err(format!(
            "open (O_DIRECTORY|O_NOFOLLOW): {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: `fd` is fresh from `open` above and not used elsewhere.
    let file = unsafe { File::from_raw_fd(fd) };

    let meta = file.metadata().map_err(|err| format!("fstat: {err}"))?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("not a real directory".to_string());
    }
    let uid = unsafe { libc::getuid() };
    if meta.uid() != uid {
        return Err(format!("owned by uid {} instead of {uid}", meta.uid()));
    }
    if meta.mode() & 0o077 != 0 {
        // SAFETY: fchmod on an fd we opened read-only is allowed and cannot race a path swap.
        let rc = unsafe { libc::fchmod(file.as_raw_fd(), 0o700) };
        if rc != 0 {
            return Err(format!("chmod 0700: {}", std::io::Error::last_os_error()));
        }
    }
    Ok(file)
}

fn open_log_file(dirfd: RawFd) -> Result<(File, u64), String> {
    // SAFETY: `dirfd` is a live directory fd; the path is a const with no NUL.
    let fd = unsafe {
        libc::openat(
            dirfd,
            c"zanemegapp.log".as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    // SAFETY: `fd` is fresh from `openat` above and not used elsewhere.
    let file = unsafe { File::from_raw_fd(fd) };

    // SAFETY: fchmod on our own fd; the mode exists before the first byte is written.
    if unsafe { libc::fchmod(fd, 0o600) } != 0 {
        return Err(format!("chmod 0600: {}", std::io::Error::last_os_error()));
    }
    let meta = file.metadata().map_err(|err| format!("fstat: {err}"))?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err("not a regular file".to_string());
    }
    let uid = unsafe { libc::getuid() };
    if meta.uid() != uid {
        return Err(format!("owned by uid {} instead of {uid}", meta.uid()));
    }
    if meta.mode() & 0o077 != 0 {
        return Err(format!("mode {:o} is not 0600", meta.mode() & 0o777));
    }
    Ok((file, meta.len()))
}

/// `1970-01-01T00:00:00Z` — UTC, no crate, no locale, no unsafe.
pub fn iso8601_utc(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    format_iso8601(secs)
}

fn format_iso8601(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let rem = unix_secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 → (year, month, day).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// `$HOME` → `~`, hostname → `<host>`, MACs → `<mac>`, IP literals → `<ip>`, then quotes and
/// control characters are escaped so one event can never produce two lines (SEC-V4-22).
pub fn redact_and_escape(text: &str) -> String {
    let mut out = text.to_string();
    let home = home_dir();
    if let Some(home) = home
        .as_deref()
        .and_then(std::path::Path::to_str)
        .filter(|home| home.len() > 1)
    {
        out = out.replace(home, "~");
    }
    if let Some(host) = hostname().filter(|host| host.len() > 2) {
        out = out.replace(&host, "<host>");
    }
    out = replace_macs(&out);
    out = replace_ip_literals(&out);
    escape_controls(&out)
}

fn escape_controls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (0x7f..=0x9f).contains(&(c as u32)) => {}
            c => out.push(c),
        }
    }
    out
}

fn replace_macs(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if is_mac_at(bytes, i) {
            out.push_str("<mac>");
            i += 17;
        } else {
            // `i` is always on a char boundary: it advances by whole characters.
            let ch = text
                .get(i..)
                .and_then(|rest| rest.chars().next())
                .unwrap_or('\u{0}');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn is_mac_at(bytes: &[u8], start: usize) -> bool {
    if start + 17 > bytes.len() {
        return false;
    }
    for (offset, byte) in bytes[start..start + 17].iter().enumerate() {
        let ok = if offset % 3 == 2 {
            *byte == b':'
        } else {
            byte.is_ascii_hexdigit()
        };
        if !ok {
            return false;
        }
    }
    true
}

/// IPv4 `a.b.c.d` and IPv6-ish literals. Runs must be contiguous, so a symbol like
/// `core::log` is never touched; IPv6 needs at least three `:` and three hex digits.
fn replace_ip_literals(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() || ch == ':' || ch == '.' {
            run.push(ch);
            continue;
        }
        push_ip_or_literal(&mut out, &run);
        run.clear();
        out.push(ch);
    }
    push_ip_or_literal(&mut out, &run);
    out
}

fn push_ip_or_literal(out: &mut String, run: &str) {
    if is_ipv4(run) || is_ipv6ish(run) {
        out.push_str("<ip>");
    } else {
        out.push_str(run);
    }
}

fn is_ipv4(run: &str) -> bool {
    let parts: Vec<&str> = run.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 3
                && part.chars().all(|c| c.is_ascii_digit())
                && part.parse::<u16>().is_ok_and(|n| n <= 255)
        })
}

fn is_ipv6ish(run: &str) -> bool {
    let colons = run.chars().filter(|c| *c == ':').count();
    let hex = run.chars().filter(char::is_ascii_hexdigit).count();
    colons >= 3 && hex >= 3 && run.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
}

/// The user's home directory from `getpwuid_r`, never from `$HOME` (design §5.13, SEC-V4-22).
pub fn home_dir() -> Option<PathBuf> {
    static HOME: OnceLock<Option<PathBuf>> = OnceLock::new();
    HOME.get_or_init(read_home_dir).clone()
}

fn read_home_dir() -> Option<PathBuf> {
    use std::ffi::CStr;
    use std::os::unix::ffi::OsStringExt as _;

    let uid = unsafe { libc::getuid() };
    let mut buf = vec![0_i8; 16_384];
    // SAFETY: `pwd` is a plain C struct; zero is a valid all-NULL value for it.
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    // SAFETY: buffer and out-pointer are valid for the call; `getpwuid_r` writes only there.
    let rc = unsafe { libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result) };
    if rc != 0 || result.is_null() || pwd.pw_dir.is_null() {
        return None;
    }
    // SAFETY: `pw_dir` points into `buf` (or the system's own storage) and is NUL-terminated.
    let dir = unsafe { CStr::from_ptr(pwd.pw_dir) };
    let path = PathBuf::from(std::ffi::OsString::from_vec(dir.to_bytes().to_vec()));
    path.is_absolute().then_some(path)
}

fn hostname() -> Option<String> {
    static HOST: OnceLock<Option<String>> = OnceLock::new();
    HOST.get_or_init(|| {
        let mut buf = [0_u8; 256];
        // SAFETY: `buf` is a valid writable buffer of the stated length.
        let rc = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
        if rc != 0 {
            return None;
        }
        let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
        String::from_utf8(buf.get(..end)?.to_vec()).ok()
    })
    .clone()
}

/// `$XDG_CONFIG_HOME/zanemegapp/logs`, defaulting to `~/.config/zanemegapp/logs`.
/// The one place a writable path is derived until `core/config.rs` lands.
pub fn log_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("zanemegapp").join("logs")
}

#[cfg(test)]
mod tests;
