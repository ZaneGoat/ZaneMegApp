use std::process::Command;

pub struct SecurityManager;

impl SecurityManager {
    /// Checks if UFW firewall (Network Isolation) is active
    pub fn is_isolation_active() -> bool {
        // Try non-interactive sudo first to check status without prompt
        if let Ok(out) = Command::new("sudo").args(["-n", "ufw", "status"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.contains("Status: active");
        }
        
        // If sudo -n failed, try reading ufw config directly
        if let Ok(content) = std::fs::read_to_string("/etc/ufw/ufw.conf") {
            return content.contains("ENABLED=yes");
        }

        false
    }

    /// Toggles UFW network isolation using pkexec
    pub fn toggle_isolation(enable: bool) -> Result<bool, String> {
        let action = if enable { "enable" } else { "disable" };
        let out = Command::new("pkexec")
            .args(["ufw", action])
            .output()
            .map_err(|e| format!("Failed to execute pkexec ufw: {e}"))?;

        if out.status.success() {
            let state_str = if enable { "Enabled" } else { "Disabled" };
            let _ = Command::new("notify-send")
                .args(["Zane Mega Center", &format!("🛡️ Network Isolation {state_str}")])
                .spawn();
            Ok(enable)
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            Err(format!("UFW command failed: {err}"))
        }
    }
}
