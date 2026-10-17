use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Clone, Debug, Default)]
pub struct SystemResources {
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_percent: f32,
    pub ram_cached_mb: f32,
    pub swap_used_gb: f32,
    pub swap_total_gb: f32,
    pub swap_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub disk_free_gb: f32,
    pub disk_percent: f32,
}

#[derive(Clone, Debug)]
pub struct CleanupModule {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub requires_root: bool,
    pub default_checked: bool,
}

pub const MODULES: &[CleanupModule] = &[
    CleanupModule {
        id: "smart_ram",
        name: "🧠 Smart RAM Optimizer",
        category: "MEMORY",
        description: "Terminates bloatware (trackers, indexers), reaps zombies, stops heavy user services",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "kernel_cache",
        name: "⚡ RAM Kernel Cache Flush",
        category: "MEMORY",
        description: "sync && echo 3 > drop_caches && sysctl vm.compact_memory=1",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "swap_reset",
        name: "🔄 Swap Memory Reset",
        category: "MEMORY",
        description: "swapoff -a && swapon -a (moves paged memory back to physical RAM)",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "service_opt",
        name: "⚙️ Background Service Optimizer",
        category: "SERVICES",
        description: "Stops non-essential system daemons (cups, ModemManager, avahi)",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "apt_cache",
        name: "📦 APT Package Cache",
        category: "STORAGE",
        description: "apt-get clean && autoclean && autoremove -y",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "pip_cache",
        name: "🐍 Python Pip Cache",
        category: "STORAGE",
        description: "pip cache purge",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "journal_logs",
        name: "📜 Systemd Journal Logs",
        category: "STORAGE",
        description: "journalctl --vacuum-size=100M",
        requires_root: true,
        default_checked: true,
    },
    CleanupModule {
        id: "user_trash",
        name: "🗑️ User Trash Bin",
        category: "STORAGE",
        description: "Removes deleted items in ~/.local/share/Trash",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "thumbnail_cache",
        name: "🖼️ Thumbnail Cache",
        category: "STORAGE",
        description: "Clears ~/.cache/thumbnails/*",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "tmp_files",
        name: "📂 /tmp Old Files",
        category: "STORAGE",
        description: "Deletes temporary files older than 24 hours",
        requires_root: false,
        default_checked: true,
    },
    CleanupModule {
        id: "flatpak_unused",
        name: "🧩 Flatpak Unused Runtimes",
        category: "CONTAINERS",
        description: "flatpak uninstall --unused -y",
        requires_root: true,
        default_checked: false,
    },
    CleanupModule {
        id: "docker_prune",
        name: "🐳 Docker Prune",
        category: "CONTAINERS",
        description: "Prunes stopped containers and dangling images",
        requires_root: true,
        default_checked: false,
    },
];

pub struct CleanerManager;

impl CleanerManager {
    /// Reads live RAM, Swap, and Root Disk metrics directly from `/proc/meminfo` and `df`
    pub fn read_system_resources() -> SystemResources {
        let mut res = SystemResources::default();

        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total_kb = 0u64;
            let mut mem_avail_kb = 0u64;
            let mut buffers_kb = 0u64;
            let mut cached_kb = 0u64;
            let mut swap_total_kb = 0u64;
            let mut swap_free_kb = 0u64;

            for line in meminfo.lines() {
                let mut parts = line.split_whitespace();
                let Some(key) = parts.next() else { continue };
                let Some(val_str) = parts.next() else { continue };
                let Ok(val) = val_str.parse::<u64>() else { continue };

                match key {
                    "MemTotal:" => mem_total_kb = val,
                    "MemAvailable:" => mem_avail_kb = val,
                    "Buffers:" => buffers_kb = val,
                    "Cached:" => cached_kb = val,
                    "SwapTotal:" => swap_total_kb = val,
                    "SwapFree:" => swap_free_kb = val,
                    _ => {}
                }
            }

            if mem_total_kb > 0 {
                let used_kb = mem_total_kb.saturating_sub(mem_avail_kb);
                res.ram_total_gb = mem_total_kb as f32 / (1024.0 * 1024.0);
                res.ram_used_gb = used_kb as f32 / (1024.0 * 1024.0);
                res.ram_percent = (used_kb as f32 / mem_total_kb as f32) * 100.0;
                res.ram_cached_mb = (buffers_kb + cached_kb) as f32 / 1024.0;
            }

            if swap_total_kb > 0 {
                let swap_used_kb = swap_total_kb.saturating_sub(swap_free_kb);
                res.swap_total_gb = swap_total_kb as f32 / (1024.0 * 1024.0);
                res.swap_used_gb = swap_used_kb as f32 / (1024.0 * 1024.0);
                res.swap_percent = (swap_used_kb as f32 / swap_total_kb as f32) * 100.0;
            }
        }

        // Read root filesystem usage via df -B1 /
        if let Ok(out) = Command::new("df").args(["-B1", "/"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let total_b = parts[1].parse::<u64>().unwrap_or(0);
                    let used_b = parts[2].parse::<u64>().unwrap_or(0);
                    let avail_b = parts[3].parse::<u64>().unwrap_or(0);
                    if total_b > 0 {
                        let gb = 1024.0 * 1024.0 * 1024.0;
                        res.disk_total_gb = total_b as f32 / gb;
                        res.disk_used_gb = used_b as f32 / gb;
                        res.disk_free_gb = avail_b as f32 / gb;
                        res.disk_percent = (used_b as f32 / total_b as f32) * 100.0;
                    }
                }
            }
        }

        res
    }

    pub fn execute_cleanup(selected_ids: &[String], tx: Sender<String>) {
        let before = Self::read_system_resources();

        tx.send("============================================================".into()).ok();
        tx.send(format!(
            "[*] STARTING CLEANUP // Pre-Scan RAM: {:.2} GB ({:.0}%) | Disk: {:.1} GB",
            before.ram_used_gb, before.ram_percent, before.disk_used_gb
        ))
        .ok();
        tx.send("============================================================".into()).ok();

        // 1. Run non-root modules immediately
        for mod_id in selected_ids {
            match mod_id.as_str() {
                "smart_ram" => Self::run_smart_ram(&tx),
                "pip_cache" => Self::run_pip_cache(&tx),
                "user_trash" => Self::run_user_trash(&tx),
                "thumbnail_cache" => Self::run_thumbnail_cache(&tx),
                "tmp_files" => Self::run_tmp_files(&tx),
                _ => {}
            }
        }

        // 2. Batch all selected root modules into a single pkexec invocation
        let mut root_script_lines: Vec<String> = Vec::new();
        for mod_id in selected_ids {
            match mod_id.as_str() {
                "kernel_cache" => {
                    root_script_lines.push("echo '[*] Executing: RAM Kernel Cache Flush...'".into());
                    root_script_lines.push("sync && echo 3 > /proc/sys/vm/drop_caches && sysctl -w vm.compact_memory=1 >/dev/null 2>&1 || true".into());
                    root_script_lines.push("echo '[✓] Kernel page cache flushed and memory compacted.'".into());
                }
                "service_opt" => {
                    root_script_lines.push("echo '[*] Executing: Background Service Optimizer...'".into());
                    root_script_lines.push("systemctl stop cups.service cups-browsed.service ModemManager.service avahi-daemon.service 2>/dev/null || true".into());
                    root_script_lines.push("echo '[✓] Non-essential system services stopped.'".into());
                }
                "swap_reset" => {
                    root_script_lines.push("echo '[*] Resetting Swap Memory...'".into());
                    root_script_lines.push("swapoff -a && swapon -a".into());
                    root_script_lines.push("echo '[✓] Swap space purged and re-enabled.'".into());
                }
                "apt_cache" => {
                    root_script_lines.push("echo '[*] Cleaning APT package cache...'".into());
                    root_script_lines.push("apt-get clean && apt-get autoclean -y && apt-get autoremove -y".into());
                    root_script_lines.push("echo '[✓] APT caches and orphaned packages cleaned.'".into());
                }
                "journal_logs" => {
                    root_script_lines.push("echo '[*] Vacuuming systemd journal logs to 100M...'".into());
                    root_script_lines.push("journalctl --vacuum-size=100M >/dev/null 2>&1 || true".into());
                    root_script_lines.push("echo '[✓] Journal vacuum complete.'".into());
                }
                "flatpak_unused" => {
                    root_script_lines.push("echo '[*] Removing unused Flatpak runtimes...'".into());
                    root_script_lines.push("flatpak uninstall --unused -y 2>/dev/null || true".into());
                    root_script_lines.push("echo '[✓] Unused Flatpak runtimes removed.'".into());
                }
                "docker_prune" => {
                    root_script_lines.push("echo '[*] Pruning unused Docker containers and images...'".into());
                    root_script_lines.push("docker container prune -f >/dev/null 2>&1; docker image prune -f >/dev/null 2>&1 || true".into());
                    root_script_lines.push("echo '[✓] Docker resources pruned.'".into());
                }
                _ => {}
            }
        }

        if !root_script_lines.is_empty() {
            tx.send("[*] Requesting root elevation (single Polkit prompt) for privileged tasks...".into()).ok();
            let combined_script = root_script_lines.join("\n");
            match Command::new("pkexec")
                .args(["bash", "-c", &combined_script])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let tx_out = tx.clone();
                        thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().map_while(Result::ok) {
                                tx_out.send(line).ok();
                            }
                        });
                    }
                    if let Some(stderr) = child.stderr.take() {
                        let tx_err = tx.clone();
                        thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines().map_while(Result::ok) {
                                if !line.trim().is_empty() {
                                    tx_err.send(format!("[!] {line}")).ok();
                                }
                            }
                        });
                    }
                    let status = child.wait();
                    if let Ok(s) = status {
                        if !s.success() {
                            tx.send("[!] Root batch exited with non-zero status (or auth cancelled).".into()).ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(format!("[ERR] Failed to spawn pkexec: {e}")).ok();
                }
            }
        }

        let after = Self::read_system_resources();
        let ram_freed_mb = ((before.ram_used_gb - after.ram_used_gb) * 1024.0).max(0.0);
        let disk_freed_mb = ((before.disk_used_gb - after.disk_used_gb) * 1024.0).max(0.0);

        tx.send("============================================================".into()).ok();
        tx.send(format!(
            "[✓] RECLAIMED // RAM Freed: {:.0} MB | Disk Freed: {:.0} MB",
            ram_freed_mb, disk_freed_mb
        ))
        .ok();
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

        let heavy_services = [
            "tracker-miner-fs-3.service",
            "tracker-extract-3.service",
            "evolution-data-server.service",
            "gvfs-daemon.service",
            "at-spi-dbus-bus.service",
        ];
        let mut stopped = 0;
        for svc in heavy_services {
            if let Ok(out) = Command::new("systemctl").args(["--user", "stop", svc]).output() {
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

    fn run_pip_cache(tx: &Sender<String>) {
        tx.send("[*] Purging Python Pip Cache...".into()).ok();
        match Command::new("pip").args(["cache", "purge"]).output() {
            Ok(o) if o.status.success() => tx.send("[✓] Pip cache purged successfully.".into()).ok(),
            Ok(o) => tx.send(format!("[!] Pip output: {}", String::from_utf8_lossy(&o.stdout).trim())).ok(),
            Err(e) => tx.send(format!("[ERR] Pip purge failed: {e}")).ok(),
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

    fn run_tmp_files(tx: &Sender<String>) {
        tx.send("[*] Cleaning /tmp files older than 24h...".into()).ok();
        let _ = Command::new("bash")
            .args(["-c", "find /tmp -type f -atime +1 -delete 2>/dev/null; true"])
            .status();
        tx.send("[✓] Old /tmp files removed.".into()).ok();
    }
}
