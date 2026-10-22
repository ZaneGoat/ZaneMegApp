mod app;
mod core;
mod theme;
mod ui;

use app::ZaneMegApp;
use eframe::egui;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, Mutex};

use core::log::{Level, Logger};

/// The one version string (D17): `Cargo.toml`, the window title and the log line all read it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "ZaneMegApp";

/// `ZaneMegApp 4.0.0` — the title, the log line and the ribbon all print this.
pub fn app_version() -> String {
    format!("{APP_NAME} {VERSION}")
}

/// The window floor (AC-05). Below this the tab bodies cannot stay legible, so the WM is told
/// not to go there; the tab bodies themselves wrap or stack at any width (L1–L6).
const MIN_INNER_SIZE: [f32; 2] = [1024.0, 640.0];

/// Symptom of the panic hook: one line, ISO-8601 UTC, version, message and a symbol-only
/// frame — no paths, no `$HOME`, no environment (AC-03, AC-47). A poisoned or contended
/// logger falls back to one `write(2)` on the fd we already hold (SEC-V4-23). It never panics:
/// every step ignores its error.
fn install_panic_hook(logger: Arc<Mutex<Logger>>) {
    let raw_fd = logger.lock().ok().and_then(|logger| logger.raw_fd());
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic payload>");
        let frame = frame_symbol(info);

        let mut written = false;
        let logger = match logger.try_lock() {
            Ok(guard) => Some(guard),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => None,
        };
        if let Some(mut logger) = logger {
            logger.write_panic(message, frame.as_deref());
            written = true;
        }
        let fallback_fd = if written { None } else { raw_fd };
        if let Some(fd) = fallback_fd {
            let line = core::log::panic_line(message, frame.as_deref());
            let bytes = line.as_bytes();
            // SAFETY: `fd` is an O_APPEND fd the logger still owns; a short write is ignored.
            unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        }
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            core::log::panic_line(message, frame.as_deref()).as_bytes(),
        );
    }));
}

/// The symbol-ish frame the panic came from: `zanemegapp::ui::tab_hardware`, never a file path
/// (AC-03's "a backtrace frame", AC-47's "no `$HOME` paths", design §12.5).
///
/// Stable `std` does not expose backtrace frames (`Backtrace::frames()` is behind the unstable
/// `backtrace_frames` feature) and D16 forbids adding the `backtrace` crate, so the frame is
/// the module path of the panic `Location` — which is also the only part of a backtrace that
/// carries no source path.
fn frame_symbol(info: &std::panic::PanicHookInfo<'_>) -> Option<String> {
    let file = info.location()?.file();
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    let parts: Vec<&str> = stem
        .split('/')
        .filter(|part| !part.is_empty() && *part != "src" && *part != ".")
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(if file.starts_with("src/") {
        format!("zanemegapp::{}", parts.join("::"))
    } else {
        parts.join("::")
    })
}

fn enforce_single_instance() -> Option<File> {
    let uid = unsafe { libc::getuid() };
    let lock_path = format!("/run/user/{uid}/zanemegapp.lock");
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .or_else(|_| {
            File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open("/tmp/zanemegapp.lock")
        })
        .ok()?;

    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if ret == 0 { Some(file) } else { None }
}

fn main() -> eframe::Result<()> {
    // First statement in `main`, before the lock, the window or any thread (AC-03).
    let logger = Arc::new(Mutex::new(Logger::open(&core::log::log_dir())));
    if let Some(reason) = logger
        .lock()
        .ok()
        .and_then(|log| log.degraded().map(str::to_string))
    {
        eprintln!("[!] ZaneMegApp: log file unusable ({reason}) — logging to stderr instead.");
    }
    let log_path = logger
        .lock()
        .ok()
        .and_then(|log| log.path())
        .map_or_else(|| "stderr".to_string(), |path| path.display().to_string());
    install_panic_hook(Arc::clone(&logger));
    if let Ok(mut log) = logger.lock() {
        log.write(
            Level::Info,
            "startup",
            &format!(
                "version=\"{}\" min_window={}x{} log=\"{log_path}\"",
                app_version(),
                MIN_INNER_SIZE[0],
                MIN_INNER_SIZE[1]
            ),
        );
    }

    // Enforce single instance lock
    let _lock_guard = match enforce_single_instance() {
        Some(guard) => guard,
        None => {
            eprintln!("[!] ZaneMegApp is already running. Focusing existing instance.");
            let _ = std::process::Command::new("notify-send")
                .args([
                    "ZaneMegApp",
                    "An instance of ZaneMegApp is already running!",
                ])
                .spawn();
            std::process::exit(0);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Must stay in sync with StartupWMClass in ZaneMegApp.desktop: this is the Wayland
            // app_id the launcher matches its window against.
            .with_app_id("ZaneMegApp")
            .with_inner_size([1140.0, 840.0])
            .with_min_inner_size(MIN_INNER_SIZE)
            .with_title(app_version()),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(ZaneMegApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-03's writer half: a panic produces exactly one line with the version, the message
    /// and a frame symbol, and the hook itself does not panic.
    #[test]
    fn panic_hook_writes_one_line_with_version_message_and_frame() {
        let dir = std::env::temp_dir().join(format!("zma-hook-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let logger = Arc::new(Mutex::new(Logger::open(&dir)));
        install_panic_hook(Arc::clone(&logger));

        let caught = std::panic::catch_unwind(|| panic!("panic hook self-test"));
        assert!(caught.is_err(), "the test panic should have been caught");

        let path = logger.lock().expect("logger").path().expect("log path");
        let text = std::fs::read_to_string(&path).expect("log file");
        assert_eq!(text.lines().count(), 1, "one line per panic: {text}");
        assert!(text.contains("PANIC"), "no PANIC level: {text}");
        assert!(text.contains("ZaneMegApp 4.0.0"), "no version: {text}");
        assert!(text.contains("panic hook self-test"), "no message: {text}");
        assert!(text.contains("zanemegapp::main"), "no frame: {text}");
        assert!(!text.contains(".rs:"), "file and line leaked: {text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
