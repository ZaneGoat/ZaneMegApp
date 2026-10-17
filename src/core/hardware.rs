use std::fs;
use std::process::Command;

pub const ISW_SECTION: &str = "16G41";

#[derive(Clone, Debug)]
pub struct HardwareTelemetry {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub nvme_temp: f32,
    #[allow(dead_code)]
    pub cpu_fan_speed: f32,
    #[allow(dead_code)]
    pub gpu_fan_speed: f32,
    pub cpu_freq_ghz: f32,
    pub cpu_governor: String,
    pub battery_pct: u32,
    pub battery_status: String,
}

impl Default for HardwareTelemetry {
    fn default() -> Self {
        Self {
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            nvme_temp: 0.0,
            cpu_fan_speed: 0.0,
            gpu_fan_speed: 0.0,
            cpu_freq_ghz: 0.0,
            cpu_governor: "powersave".into(),
            battery_pct: 100,
            battery_status: "AC".into(),
        }
    }
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

    /// Read instantaneous thermal, clock, NVMe, and battery telemetry from sysfs
    pub fn read_telemetry() -> HardwareTelemetry {
        let mut telem = HardwareTelemetry::default();

        // 1. Scan /sys/class/hwmon for coretemp, nvme, and GPU sensors
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = fs::read_to_string(p.join("name")) {
                    let name = name.trim();
                    if name == "coretemp" {
                        if let Ok(t) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(raw) = t.trim().parse::<f32>() {
                                telem.cpu_temp = raw / 1000.0;
                            }
                        }
                    } else if name == "nvme" {
                        if let Ok(t) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(raw) = t.trim().parse::<f32>() {
                                telem.nvme_temp = raw / 1000.0;
                            }
                        }
                    } else if name.contains("nouveau") || name.contains("nvidia") || name.contains("amdgpu") {
                        if let Ok(t) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(raw) = t.trim().parse::<f32>() {
                                telem.gpu_temp = raw / 1000.0;
                            }
                        }
                    }
                }
            }
        }

        // Fallback CPU temp from thermal_zone0
        if telem.cpu_temp == 0.0 {
            if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
                if let Ok(raw) = temp_str.trim().parse::<f32>() {
                    telem.cpu_temp = raw / 1000.0;
                }
            }
        }

        // Fallback secondary thermal zone for GPU / PCH
        if telem.gpu_temp == 0.0 {
            if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone1/temp") {
                if let Ok(raw) = temp_str.trim().parse::<f32>() {
                    telem.gpu_temp = raw / 1000.0;
                }
            }
        }

        // 2. Read CPU Frequency & Governor
        if let Ok(freq_str) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
            if let Ok(khz) = freq_str.trim().parse::<f32>() {
                telem.cpu_freq_ghz = khz / 1_000_000.0;
            }
        }
        if let Ok(gov_str) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
            telem.cpu_governor = gov_str.trim().to_uppercase();
        }

        // 3. Read Battery Capacity & Status
        if let Ok(cap_str) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
            if let Ok(cap) = cap_str.trim().parse::<u32>() {
                telem.battery_pct = cap;
            }
        }
        if let Ok(stat_str) = fs::read_to_string("/sys/class/power_supply/BAT1/status") {
            telem.battery_status = stat_str.trim().to_string();
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
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.1").status();
                let _ = Command::new("rfkill").args(["unblock", "bluetooth"]).status();
                let _ = Command::new("pkexec")
                    .args(["bash", "-c", "tlp ac >/dev/null 2>&1; isw -b on"])
                    .status();
                let _ = Command::new("notify-send")
                    .args(["ZaneMegApp", "🔥 EXTREME Mode Active (144Hz + Cooler Boost)"])
                    .spawn();
                Ok("Applied EXTREME profile: 144Hz, Full Power, Bluetooth ON, Cooler Boost ON".into())
            }
            "balance" => {
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.2").status();
                let _ = Command::new("rfkill").args(["unblock", "bluetooth"]).status();
                let _ = Command::new("pkexec")
                    .args(["bash", "-c", "tlp start >/dev/null 2>&1; isw -b off"])
                    .status();
                let _ = Command::new("notify-send")
                    .args(["ZaneMegApp", "⚖️ BALANCE Mode Active (48Hz + Auto Fans)"])
                    .spawn();
                Ok("Applied BALANCE profile: 48Hz, Smart Fans, Daily Power".into())
            }
            "battery" => {
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.2").status();
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
