import subprocess

class SecurityManager:
    """Manages system isolation, UFW firewall state."""
    
    def __init__(self, log_callback=None):
        self.log = log_callback if log_callback else print

    def is_isolation_active(self) -> bool:
        """Checks if UFW is active."""
        try:
            out = subprocess.check_output(["sudo", "-n", "ufw", "status"], text=True, stderr=subprocess.DEVNULL)
            return "Status: active" in out
        except Exception:
            return False

    def toggle_isolation(self, enable: bool) -> bool:
        """Enables or disables the UFW firewall via pkexec."""
        cmd = "enable" if enable else "disable"
        try:
            r = subprocess.run(["pkexec", "ufw", cmd], capture_output=True, text=True)
            if r.returncode == 0:
                state_str = "Enabled" if enable else "Disabled"
                subprocess.run(["notify-send", "Zane Mega Center", f"🛡️ Network Isolation {state_str}"])
                return True
            else:
                self.log(f"Failed to toggle firewall: {r.stderr}")
                return False
        except Exception as e:
            self.log(f"Exception toggling firewall: {e}")
            return False
