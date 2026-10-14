import subprocess
import sys
from pathlib import Path

class BluetoothManager:
    """Manages RFCOMM headsets."""
    
    def __init__(self, log_callback=None):
        self.log = log_callback if log_callback else print
        self.script_path = Path.home() / "soundpeats_controller.py"

    def set_headset_mode(self, mode_id: str):
        """Spawns the external python controller script for the headset."""
        mac_addr = "41:42:F0:AC:0F:EF"
        
        self.log(f"[*] Initializing RFCOMM to SOUNDPEATS Q3 Pro ({mac_addr}) for {mode_id}...")
        try:
            p = subprocess.Popen(
                [sys.executable, str(self.script_path), mac_addr, mode_id],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
            )
            for line in iter(p.stdout.readline, ''):
                if line:
                    self.log(line.strip())
            p.wait()
            return p.returncode == 0
        except Exception as e:
            self.log(f"[ERR] Subprocess failed: {e}")
            return False
