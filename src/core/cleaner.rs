use std::fs;
use std::process::Command;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct CleanupModule {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub requires_root: bool,
    pub default_checked: bool,
}

pub const MODULES: &[CleanupModule] = &[
    CleanupModule {
        id: "smart_ram",
        name: "🧠 Smart RAM Optimizer",
        description: "Terminates bloatware (trackers, indexers), reaps zombies, stops heavy user services",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "kernel_cache",
        name: "🧠 RAM Kernel Cache Flush",
        description: "sync && echo 3 > drop_caches && sysctl vm.compact_memory=1",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "service_opt",
        name: "⚙️ Background Service Optimizer",
        description: "Stops non-essential system services (cups, ModemManager, avahi)",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "swap_reset",
        name: "Swap Memory Reset",
        description: "swapoff -a && swapon -a",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "apt_cache",
        name: "APT Package Cache",
        description: "apt-get clean && autoclean && autoremove",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "pip_cache",
        name: "Python Pip Cache",
        description: "pip cache purge",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "journal_logs",
        name: "Systemd Journal Logs",
        description: "journalctl --vacuum-size=100M",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "user_trash",
        name: "User Trash Bin",
        description: "Removes deleted items in ~/.local/share/Trash",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "thumbnail_cache",
        name: "Thumbnail Cache",
        description: "Clears ~/.cache/thumbnails/*",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "flatpak_unused",
        name: "Flatpak Unused",
        description: "flatpak uninstall --unused -y",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "docker_prune",
        name: "Docker Prune",
        description: "Prunes stopped containers and dangling images",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "tmp_files",
        name: "/tmp Old Files",
        description: "Deletes temporary files older than 24 hours",
        requires_root: false,
        default_checked: true,
    },
];

pub struct CleanerManager;

impl CleanerManager {
    pub fn execute_cleanup(selected_ids: &[String], tx: Sender<String>) {
        tx.send("============================================================".into()).ok();
        tx.send("[*] STARTING HIGH-EFFICIENCY SYSTEM CLEANUP PROTOCOL".into()).ok();
        tx.send("============================================================".into()).ok();

        for mod_id in selected_ids {
            match mod_id.as_str() {
                "smart_ram" => Self::run_smart_ram(&tx),
                "kernel_cache" => Self::run_kernel_cache(&tx),
                "service_opt" => Self::run_service_opt(&tx),
                "swap_reset" => Self::run_swap_reset(&tx),
                "apt_cache" => Self::run_apt_cache(&tx),
                "pip_cache" => Self::run_pip_cache(&tx),
                "journal_logs" => Self::run_journal_logs(&tx),
                "user_trash" => Self::run_user_trash(&tx),
                "thumbnail_cache" => Self::run_thumbnail_cache(&tx),
                "flatpak_unused" => Self::run_flatpak_unused(&tx),
                "docker_prune" => Self::run_docker_prune(&tx),
                "tmp_files" => Self::run_tmp_files(&tx),
                other => {
                    tx.send(format!("[!] Unknown module: {other}")).ok();
                }
            }
        }

        tx.send("============================================================".into()).ok();
        tx.send("[✓] SYSTEM CLEANUP PROTOCOL COMPLETED".into()).ok();
        tx.send("============================================================".into()).ok();
    }

    fn run_smart_ram(tx: &Sender<String>) {
        tx.send("[*] Executing: Smart RAM Optimizer...".into()).ok();

        let bloat_patterns = [
            "tracker-miner", "tracker-extract", "tracker-store",
            "evolution-data", "evolution-calendar", "evolution-address",
            "goa-daemon", "zeitgeist", "baloo_file", "akonadi_",
            "gnome-software", "packagekitd", "update-notifier",
            "gsd-housekeeping", "at-spi-bus", "xdg-desktop-portal",
            "tumblerd",
        ];

        let mut killed_count = 0;
        let my_pid = std::process::id();

        // Scan /proc for bloatware processes
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        if pid == my_pid || pid == 1 {
                            continue;
                        }
                        let comm_path = entry.path().join("comm");
                        if let Ok(comm) = fs::read_to_string(comm_path) {
                            let comm = comm.trim();
                            if bloat_patterns.iter().any(|pat| comm.contains(pat)) {
                                let _ = Command::new("kill").args(["-15", &pid.to_string()]).status();
                                tx.send(format!("  → Sent SIGTERM to {comm} (PID {pid})")).ok();
                                killed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if killed_count > 0 {
            tx.send(format!("[RAM] Terminated {killed_count} background bloat processes.")).ok();
        } else {
            tx.send("[RAM] No background bloat processes found running.".into()).ok();
        }

        // Stop heavy user services
        let heavy_services = [
            "tracker-miner-fs-3.service",
            "tracker-extract-3.service",
            "evolution-data-server.service",
            "gvfs-daemon.service",
            "at-spi-dbus-bus.service",
        ];
        let mut stopped = 0;
        for svc in heavy_services {
            let res = Command::new("systemctl")
                .args(["--user", "stop", svc])
                .output();
            if let Ok(out) = res {
                if out.status.success() {
                    stopped += 1;
                }
            }
        }
        if stopped > 0 {
            tx.send(format!("[RAM] Stopped {stopped} heavy user services.")).ok();
        }
        tx.send("[✓] Smart RAM Optimizer finished.".into()).ok();
    }

    fn run_kernel_cache(tx: &Sender<String>) {
        tx.send("[*] Executing: RAM Kernel Cache Flush...".into()).ok();
        let cmd = "sync && echo 3 > /proc/sys/vm/drop_caches && sysctl -w vm.compact_memory=1";
        let out = Command::new("pkexec")
            .args(["bash", "-c", cmd])
            .output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Kernel page cache flushed and memory compacted.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Error flushing cache: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] Failed to execute pkexec: {e}")).ok(),
        };
    }

    fn run_service_opt(tx: &Sender<String>) {
        tx.send("[*] Executing: Background Service Optimizer...".into()).ok();
        let services = ["cups.service", "cups-browsed.service", "ModemManager.service", "avahi-daemon.service"];
        for svc in services {
            let _ = Command::new("pkexec").args(["systemctl", "stop", svc]).output();
        }
        tx.send("[✓] Non-essential system services stopped.".into()).ok();
    }

    fn run_swap_reset(tx: &Sender<String>) {
        tx.send("[*] Resetting Swap Memory...".into()).ok();
        let out = Command::new("pkexec").args(["bash", "-c", "swapoff -a && swapon -a"]).output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Swap space purged and re-enabled.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Swap reset output: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] Swap reset failed: {e}")).ok(),
        };
    }

    fn run_apt_cache(tx: &Sender<String>) {
        tx.send("[*] Cleaning APT package cache...".into()).ok();
        let out = Command::new("pkexec")
            .args(["bash", "-c", "apt-get clean && apt-get autoclean && apt-get autoremove -y"])
            .output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] APT caches and orphaned packages cleaned.".into()).ok(),
            Ok(o) => tx.send(format!("[!] APT output: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] APT clean failed: {e}")).ok(),
        };
    }

    fn run_pip_cache(tx: &Sender<String>) {
        tx.send("[*] Purging Python Pip Cache...".into()).ok();
        let out = Command::new("pip").args(["cache", "purge"]).output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Pip cache purged successfully.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Pip output: {}", String::from_utf8_lossy(&o.stdout))).ok(),
            Err(e) => tx.send(format!("[ERR] Pip purge failed: {e}")).ok(),
        };
    }

    fn run_journal_logs(tx: &Sender<String>) {
        tx.send("[*] Vacuuming systemd journal logs to 100M...".into()).ok();
        let out = Command::new("pkexec").args(["journalctl", "--vacuum-size=100M"]).output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Journal vacuum complete.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Journal output: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] Journalctl failed: {e}")).ok(),
        };
    }

    fn run_user_trash(tx: &Sender<String>) {
        tx.send("[*] Emptying user trash bin...".into()).ok();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/zane".into());
        let _ = Command::new("bash")
            .args(["-c", &format!("rm -rf {home}/.local/share/Trash/files/* {home}/.local/share/Trash/info/*")])
            .status();
        tx.send("[✓] Trash bin emptied.".into()).ok();
    }

    fn run_thumbnail_cache(tx: &Sender<String>) {
        tx.send("[*] Cleaning thumbnail cache...".into()).ok();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/zane".into());
        let _ = Command::new("bash")
            .args(["-c", &format!("rm -rf {home}/.cache/thumbnails/* {home}/.thumbnails/*")])
            .status();
        tx.send("[✓] Thumbnail cache purged.".into()).ok();
    }

    fn run_flatpak_unused(tx: &Sender<String>) {
        tx.send("[*] Removing unused Flatpak runtimes...".into()).ok();
        let out = Command::new("pkexec").args(["flatpak", "uninstall", "--unused", "-y"]).output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Unused Flatpak runtimes removed.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Flatpak output: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] Flatpak failed: {e}")).ok(),
        };
    }

    fn run_docker_prune(tx: &Sender<String>) {
        tx.send("[*] Pruning unused Docker containers and images...".into()).ok();
        let out = Command::new("pkexec")
            .args(["bash", "-c", "docker container prune -f && docker image prune -f"])
            .output();
        match out {
            Ok(o) if o.status.success() => tx.send("[✓] Docker resources pruned.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Docker output: {}", String::from_utf8_lossy(&o.stderr))).ok(),
            Err(e) => tx.send(format!("[ERR] Docker prune failed: {e}")).ok(),
        };
    }

    fn run_tmp_files(tx: &Sender<String>) {
        tx.send("[*] Cleaning /tmp files older than 24h...".into()).ok();
        let _ = Command::new("bash")
            .args(["-c", "find /tmp -type f -atime +1 -delete 2>/dev/null; true"])
            .status();
        tx.send("[✓] Old /tmp files removed.".into()).ok();
    }
}
