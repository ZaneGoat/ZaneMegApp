use std::fs;
use std::path::Path;
use std::process::Command;

pub const ISW_SECTION: &str = "16G41";

#[derive(Clone, Debug, Default)]
pub struct CpuTimes {
    pub total: u64,
    pub idle: u64,
}

#[derive(Clone, Debug)]
pub struct HardwareTelemetry {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub nvme_temp: f32,
    pub cpu_load_pct: f32,
    pub core_loads: Vec<f32>,
    #[allow(dead_code)]
    pub cpu_fan_speed: f32,
    #[allow(dead_code)]
    pub gpu_fan_speed: f32,
    pub cpu_freq_ghz: f32,
    pub cpu_governor: String,
    pub battery_pct: u32,
    pub battery_status: String,
    pub uptime_formatted: String,
}

impl Default for HardwareTelemetry {
    fn default() -> Self {
        Self {
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            nvme_temp: 0.0,
            cpu_load_pct: 0.0,
            core_loads: vec![0.0; 12],
            cpu_fan_speed: 0.0,
            gpu_fan_speed: 0.0,
            cpu_freq_ghz: 0.0,
            cpu_governor: "POWERSAVE".into(),
            battery_pct: 100,
            battery_status: "AC".into(),
            uptime_formatted: "0h 00m".into(),
        }
    }
}

/// A hwmon `temp*_input` or thermal-zone file, in °C. Both report millidegrees.
fn temperature_c(path: &Path) -> Option<f32> {
    fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f32>()
        .ok()
        .map(|millidegrees| millidegrees / 1000.0)
}

/// The hwmon sensor names that carry the discrete GPU's temperature, across nouveau, the
/// proprietary NVIDIA driver and amdgpu.
fn is_gpu_sensor(name: &str) -> bool {
    name.contains("nouveau") || name.contains("nvidia") || name.contains("amdgpu")
}

pub struct HardwareManager;

impl HardwareManager {
    /// Run an ISW command with pkexec for root privilege without a TTY
    pub fn run_isw(args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new("pkexec");
        cmd.arg("isw");
        for arg in args {
            cmd.arg(arg);
        }
        match cmd.output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if out.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!(
                        "ISW Error (exit {}): {}",
                        out.status.code().unwrap_or(-1),
                        stderr.trim()
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute pkexec isw: {e}")),
        }
    }

    /// Reads raw CPU jiffies from `/proc/stat` for overall CPU + per-thread cores
    pub fn read_cpu_times() -> (CpuTimes, Vec<CpuTimes>) {
        let mut overall = CpuTimes::default();
        let mut cores = Vec::new();

        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            for line in stat.lines() {
                if !line.starts_with("cpu") {
                    break;
                }
                let mut parts = line.split_whitespace();
                let Some(label) = parts.next() else { continue };
                let vals: Vec<u64> = parts.filter_map(|s| s.parse::<u64>().ok()).collect();
                if vals.len() >= 4 {
                    let idle = vals[3] + vals.get(4).copied().unwrap_or(0);
                    let total: u64 = vals.iter().sum();
                    if label == "cpu" {
                        overall = CpuTimes { total, idle };
                    } else {
                        cores.push(CpuTimes { total, idle });
                    }
                }
            }
        }
        (overall, cores)
    }

    /// Read instantaneous thermal, clock, NVMe, battery, and CPU load telemetry from sysfs/procfs
    pub fn read_telemetry(
        prev_overall: &mut CpuTimes,
        prev_cores: &mut Vec<CpuTimes>,
    ) -> HardwareTelemetry {
        let mut telem = HardwareTelemetry::default();

        // 0. Compute CPU load % deltas from /proc/stat
        let (cur_overall, cur_cores) = Self::read_cpu_times();
        if prev_overall.total > 0 && cur_overall.total > prev_overall.total {
            let d_total = (cur_overall.total - prev_overall.total) as f32;
            let d_idle = (cur_overall.idle.saturating_sub(prev_overall.idle)) as f32;
            telem.cpu_load_pct = ((1.0 - d_idle / d_total) * 100.0).clamp(0.0, 100.0);
        }
        *prev_overall = cur_overall;

        if prev_cores.len() == cur_cores.len() && !cur_cores.is_empty() {
            telem.core_loads = cur_cores
                .iter()
                .zip(prev_cores.iter())
                .map(|(cur, prev)| {
                    if cur.total > prev.total {
                        let dt = (cur.total - prev.total) as f32;
                        let di = (cur.idle.saturating_sub(prev.idle)) as f32;
                        ((1.0 - di / dt) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    }
                })
                .collect();
        } else if !cur_cores.is_empty() {
            telem.core_loads = vec![0.0; cur_cores.len()];
        }
        *prev_cores = cur_cores;

        // 1. Scan /sys/class/hwmon for coretemp, nvme, and GPU sensors
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(name) = fs::read_to_string(path.join("name")) {
                    // Which reading this sensor feeds. Anything else in hwmon is not ours and is
                    // skipped before its temp file is read.
                    let slot = match name.trim() {
                        "coretemp" => &mut telem.cpu_temp,
                        "nvme" => &mut telem.nvme_temp,
                        name if is_gpu_sensor(name) => &mut telem.gpu_temp,
                        _ => continue,
                    };
                    // A sensor that will not read keeps its previous value rather than zeroing it.
                    if let Some(celsius) = temperature_c(&path.join("temp1_input")) {
                        *slot = celsius;
                    }
                }
            }
        }

        // Fallback CPU temp from thermal_zone0
        if telem.cpu_temp == 0.0
            && let Some(celsius) = temperature_c(Path::new("/sys/class/thermal/thermal_zone0/temp"))
        {
            telem.cpu_temp = celsius;
        }

        // Fallback secondary thermal zone for GPU / PCH
        if telem.gpu_temp == 0.0
            && let Some(celsius) = temperature_c(Path::new("/sys/class/thermal/thermal_zone1/temp"))
        {
            telem.gpu_temp = celsius;
        }

        // 2. Read CPU Frequency & Governor
        if let Ok(freq_str) =
            fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
            && let Ok(khz) = freq_str.trim().parse::<f32>()
        {
            telem.cpu_freq_ghz = khz / 1_000_000.0;
        }
        if let Ok(gov_str) =
            fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        {
            telem.cpu_governor = gov_str.trim().to_uppercase();
        }

        // 3. Read Battery Capacity & Status
        if let Ok(cap_str) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
            && let Ok(cap) = cap_str.trim().parse::<u32>()
        {
            telem.battery_pct = cap;
        }
        if let Ok(stat_str) = fs::read_to_string("/sys/class/power_supply/BAT1/status") {
            telem.battery_status = stat_str.trim().to_string();
        }

        // 4. Read System Uptime from /proc/uptime
        if let Ok(up_str) = fs::read_to_string("/proc/uptime")
            && let Some(first) = up_str.split_whitespace().next()
            && let Ok(secs_f) = first.parse::<f64>()
        {
            let total_secs = secs_f as u64;
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            telem.uptime_formatted = format!("{hours}h {mins:02}m");
        }

        telem
    }

    pub fn set_cooler_boost(enable: bool) -> Result<String, String> {
        let val = if enable { "on" } else { "off" };
        Self::run_isw(&["-b", val])?;
        Ok(format!(
            "Cooler Boost set to {}",
            if enable { "ON (MAX RPM)" } else { "OFF (AUTO)" }
        ))
    }

    pub fn set_battery_limit(pct: u32) -> Result<String, String> {
        let val = pct.to_string();
        Self::run_isw(&["-t", &val])?;
        Ok(format!("Battery charge threshold capped at {pct}%"))
    }

    pub fn apply_custom_fan_curve() -> Result<String, String> {
        Self::run_isw(&["-w", ISW_SECTION])?;
        Ok(format!("Applied ISW custom fan curve ({ISW_SECTION})"))
    }

    pub fn reset_fans_auto() -> Result<String, String> {
        Self::run_isw(&["-b", "off"])?;
        Ok("Fans reset to BIOS Auto curve".into())
    }

    pub fn apply_power_profile(profile: &str) -> Result<String, String> {
        match profile {
            "extreme" => {
                let _ = Command::new("kscreen-doctor")
                    .arg("output.1.mode.1")
                    .status();
                let _ = Command::new("rfkill")
                    .args(["unblock", "bluetooth"])
                    .status();
                let _ = Command::new("pkexec")
                    .args(["bash", "-c", "tlp ac >/dev/null 2>&1; isw -b on"])
                    .status();
                let _ = Command::new("notify-send")
                    .args([
                        "ZaneMegApp",
                        "🔥 EXTREME Mode Active (144Hz + Cooler Boost)",
                    ])
                    .spawn();
                Ok(
                    "Applied EXTREME profile: 144Hz, Full Power, Bluetooth ON, Cooler Boost ON"
                        .into(),
                )
            }
            "balance" => {
                let _ = Command::new("kscreen-doctor")
                    .arg("output.1.mode.2")
                    .status();
                let _ = Command::new("rfkill")
                    .args(["unblock", "bluetooth"])
                    .status();
                let _ = Command::new("pkexec")
                    .args(["bash", "-c", "tlp start >/dev/null 2>&1; isw -b off"])
                    .status();
                let _ = Command::new("notify-send")
                    .args(["ZaneMegApp", "⚖️ BALANCE Mode Active (48Hz + Auto Fans)"])
                    .spawn();
                Ok("Applied BALANCE profile: 48Hz, Smart Fans, Daily Power".into())
            }
            "battery" => {
                let _ = Command::new("kscreen-doctor")
                    .arg("output.1.mode.2")
                    .status();
                let _ = Command::new("rfkill").args(["block", "bluetooth"]).status();
                let _ = Command::new("pkexec")
                    .args(["bash", "-c", "tlp bat >/dev/null 2>&1; isw -b off"])
                    .status();
                let _ = Command::new("notify-send")
                    .args(["ZaneMegApp", "🔋 SUPER BATTERY Active (48Hz + BT Off)"])
                    .spawn();
                Ok("Applied SUPER BATTERY profile: 48Hz, Silent Fans, Bluetooth OFF".into())
            }
            _ => Err(format!("Unknown profile: {profile}")),
        }
    }
}
