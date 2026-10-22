use std::collections::BTreeSet;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct NetInterface {
    pub name: String,
    pub state: String,
    pub ip_cidr: String,
    pub clean_ip: String,
}

pub struct SecurityManager;

impl SecurityManager {
    /// Checks if UFW firewall (Network Isolation) is active
    pub fn is_isolation_active() -> bool {
        if let Ok(out) = Command::new("sudo").args(["-n", "ufw", "status"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.contains("Status: active");
        }

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
                .args(["ZaneMegApp", &format!("🛡️ Network Isolation {state_str}")])
                .spawn();
            Ok(enable)
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            Err(format!("UFW command failed: {}", err.trim()))
        }
    }

    /// Reads active IPv4 network interfaces for Pentest LHOST quick-reference
    pub fn read_interfaces() -> Vec<NetInterface> {
        let mut list = Vec::new();
        if let Ok(out) = Command::new("ip")
            .args(["-4", "-br", "addr", "show"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    if name == "lo" {
                        continue;
                    }
                    let state = parts[1].to_string();
                    let ip_cidr = parts[2].to_string();
                    let clean_ip = ip_cidr.split('/').next().unwrap_or(&ip_cidr).to_string();
                    list.push(NetInterface {
                        name,
                        state,
                        ip_cidr,
                        clean_ip,
                    });
                }
            }
        }
        list
    }

    /// Reads active listening TCP ports on the machine via `ss -tlnH`
    pub fn read_listening_ports() -> Vec<u16> {
        let mut ports = BTreeSet::new();
        if let Ok(out) = Command::new("ss").args(["-tlnH"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local_addr = parts[3];
                    if let Some(port_str) = local_addr.rsplit(':').next()
                        && let Ok(p) = port_str.parse::<u16>()
                    {
                        ports.insert(p);
                    }
                }
            }
        }
        ports.into_iter().collect()
    }

    /// Re-injects the Waydroid Docker/UFW network bridge override script
    pub fn inject_waydroid_bridge() -> Result<String, String> {
        let script = "/usr/local/bin/waydroid-net-override.sh";
        let out = Command::new("pkexec")
            .args(["/bin/bash", script])
            .output()
            .map_err(|e| format!("Failed to execute pkexec waydroid-net-override.sh: {e}"))?;

        if out.status.success() {
            let _ = Command::new("notify-send")
                .args([
                    "ZaneMegApp",
                    "🤖 Waydroid Docker/NAT Bridge Rules Injected!",
                ])
                .spawn();
            Ok("Waydroid DOCKER-USER & NAT MASQUERADE rules injected".into())
        } else {
            Err(format!(
                "Bridge script error: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
}
