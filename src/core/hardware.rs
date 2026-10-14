use std::fs;
use std::process::Command;

pub const ISW_SECTION: &str = "16G41";

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct HardwareTelemetry {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub cpu_fan_speed: f32,
    pub gpu_fan_speed: f32,
    pub cooler_boost: bool,
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
                    Err(format!("ISW Error (exit {}): {}", out.status.code().unwrap_or(-1), stderr))
                }
            }
            Err(e) => Err(format!("Failed to execute pkexec isw: {e}")),
        }
    }

    /// Read thermal telemetry from ISW or sysfs fallback
    pub fn read_telemetry() -> HardwareTelemetry {
        let mut telem = HardwareTelemetry::default();

        // 1. Try reading directly from sysfs first for non-blocking instantaneous read
        if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            if let Ok(raw) = temp_str.trim().parse::<f32>() {
                telem.cpu_temp = raw / 1000.0;
            }
        }

        // Try reading GPU or secondary hwmon thermal zones
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = fs::read_to_string(p.join("name")) {
                    let name = name.trim();
                    if name.contains("nouveau") || name.contains("nvidia") || name.contains("amdgpu") {
                        if let Ok(temp_str) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(raw) = temp_str.trim().parse::<f32>() {
                                telem.gpu_temp = raw / 1000.0;
                            }
                        }
                    }
                }
            }
        }

        // If GPU temp wasn't found in hwmon, approximate or read secondary thermal zone
        if telem.gpu_temp == 0.0 {
            if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone1/temp") {
                if let Ok(raw) = temp_str.trim().parse::<f32>() {
                    telem.gpu_temp = raw / 1000.0;
                }
            }
        }

        // If sysfs didn't provide temps, attempt ISW dump
        if telem.cpu_temp == 0.0 {
            if let Ok(out) = Self::run_isw(&["-r", "1"]) {
                for line in out.lines() {
                    if line.contains("CPU") && line.contains("°C") {
                        if let Some(t) = extract_number(line) {
                            telem.cpu_temp = t;
                        }
                    } else if line.contains("GPU") && line.contains("°C") {
                        if let Some(t) = extract_number(line) {
                            telem.gpu_temp = t;
                        }
                    } else if (line.contains("Fan 1") || line.contains("CPU Fan")) && line.contains('%') {
                        if let Some(s) = extract_number(line) {
                            telem.cpu_fan_speed = s;
                        }
                    } else if (line.contains("Fan 2") || line.contains("GPU Fan")) && line.contains('%') {
                        if let Some(s) = extract_number(line) {
                            telem.gpu_fan_speed = s;
                        }
                    }
                }
            }
        }

        telem
    }

    pub fn set_cooler_boost(enable: bool) -> Result<String, String> {
        let val = if enable { "on" } else { "off" };
        Self::run_isw(&["-b", val])
    }

    pub fn set_battery_limit(pct: u32) -> Result<String, String> {
        let val = pct.to_string();
        Self::run_isw(&["-t", &val])
    }

    pub fn apply_custom_fan_curve() -> Result<String, String> {
        Self::run_isw(&["-w", ISW_SECTION])
    }

    pub fn reset_fans_auto() -> Result<String, String> {
        Self::run_isw(&["-b", "off"])
    }

    pub fn apply_power_profile(profile: &str) -> Result<String, String> {
        match profile {
            "extreme" => {
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.1").status();
                let _ = Command::new("rfkill").args(["unblock", "bluetooth"]).status();
                let _ = Command::new("pkexec").args(["tlp", "ac"]).status();
                let _ = Self::set_cooler_boost(true);
                Ok("Applied EXTREME profile: 144Hz, Full Power, Bluetooth On, Cooler Boost ON".into())
            }
            "balance" => {
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.2").status();
                let _ = Command::new("rfkill").args(["unblock", "bluetooth"]).status();
                let _ = Command::new("pkexec").args(["tlp", "start"]).status();
                let _ = Self::set_cooler_boost(false);
                Ok("Applied BALANCE profile: 48Hz, Smart Fans, Daily Power".into())
            }
            "battery" => {
                let _ = Command::new("kscreen-doctor").arg("output.1.mode.2").status();
                let _ = Command::new("rfkill").args(["block", "bluetooth"]).status();
                let _ = Command::new("pkexec").args(["tlp", "bat"]).status();
                let _ = Self::set_cooler_boost(false);
                Ok("Applied SUPER BATTERY profile: 48Hz, Silent Fans, Bluetooth Off".into())
            }
            _ => Err(format!("Unknown profile: {profile}")),
        }
    }
}

fn extract_number(s: &str) -> Option<f32> {
    let mut num = String::new();
    let mut in_num = false;
    for c in s.chars() {
        if c.is_ascii_digit() || (c == '.' && in_num) {
            num.push(c);
            in_num = true;
        } else if in_num {
            break;
        }
    }
    num.parse::<f32>().ok()
}
