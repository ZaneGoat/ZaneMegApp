mod app;
mod core;
mod theme;
mod ui;

use app::ZaneMegApp;
use eframe::egui;
use std::fs::File;
use std::os::unix::io::AsRawFd;

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
    if ret == 0 {
        Some(file)
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    // Enforce single instance lock
    let _lock_guard = match enforce_single_instance() {
        Some(guard) => guard,
        None => {
            eprintln!("[!] ZaneMegApp is already running. Focusing existing instance.");
            let _ = std::process::Command::new("notify-send")
                .args(["ZaneMegApp", "An instance of ZaneMegApp is already running!"])
                .spawn();
            std::process::exit(0);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1140.0, 840.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("⚡ ZANE MEGAPP // V3.0 VISUAL COMMAND DECK"),
        ..Default::default()
    };

    eframe::run_native(
        "ZaneMegApp",
        native_options,
        Box::new(|cc| Ok(Box::new(ZaneMegApp::new(cc)))),
    )
}
