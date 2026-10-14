import subprocess
import threading
import re

ISW_SECTION = "16G41"

class HardwareManager:
    """Manages ISW fan control, battery limits, and system power profiles."""
    
    def __init__(self, log_callback=None):
        self.log = log_callback if log_callback else print

    def run_isw(self, *args):
        try:
            cmd = ["pkexec", "isw"] + list(args)
            p = subprocess.run(cmd, capture_output=True, text=True)
            return p.stdout, p.stderr, p.returncode
        except Exception as e:
            self.log(f"ISW Execution Failed: {e}")
            return "", str(e), 1

    def set_cooler_boost(self, state: bool):
        val = "on" if state else "off"
        self.log(f"Setting cooler boost: {val.upper()}")
        out, err, rc = self.run_isw("-b", val)
        return rc == 0

    def set_battery_limit(self, limit_pct: int):
        self.log(f"Setting battery charge limit to {limit_pct}%...")
        out, err, rc = self.run_isw("-t", str(limit_pct))
        return rc == 0

    def get_telemetry(self):
        """Reads CPU/GPU temps and fan speeds from ISW."""
        out, err, rc = self.run_isw("-r", "1")
        data = {}
        if rc == 0 and out.strip():
            # Minimal parsing based on known ISW output formatting
            for line in out.splitlines():
                if "CPU" in line and "°C" in line:
                    # E.g., CPU: 45°C
                    match = re.search(r"CPU.*?(\d+)", line)
                    if match: data["cpu_t"] = match.group(1)
                if "GPU" in line and "°C" in line:
                    match = re.search(r"GPU.*?(\d+)", line)
                    if match: data["gpu_t"] = match.group(1)
                if "Fan 1" in line or "CPU Fan" in line:
                    match = re.search(r"(\d+)\s*%", line)
                    if match: data["cpu_fs"] = match.group(1)
                if "Fan 2" in line or "GPU Fan" in line:
                    match = re.search(r"(\d+)\s*%", line)
                    if match: data["gpu_fs"] = match.group(1)
        return data

    def set_power_profile(self, profile: str):
        """Supported profiles: extreme, balance, battery"""
        self.log(f"Applying Power Profile: {profile.upper()}")
        try:
            if profile == "extreme":
                subprocess.run(["kscreen-doctor", "output.1.mode.1"])
                subprocess.run(["rfkill", "unblock", "bluetooth"])
                subprocess.run(["sudo", "tlp", "ac"])
                self.set_cooler_boost(True)
            elif profile == "balance":
                subprocess.run(["kscreen-doctor", "output.1.mode.2"])
                subprocess.run(["rfkill", "unblock", "bluetooth"])
                subprocess.run(["sudo", "tlp", "start"])
                self.set_cooler_boost(False)
            elif profile == "battery":
                subprocess.run(["kscreen-doctor", "output.1.mode.2"])
                subprocess.run(["rfkill", "block", "bluetooth"])
                subprocess.run(["sudo", "tlp", "bat"])
                self.set_cooler_boost(False)
            return True
        except Exception as e:
            self.log(f"Failed to set power profile: {e}")
            return False
