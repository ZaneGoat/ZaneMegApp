import os
import signal
import subprocess
import psutil
import json
from pathlib import Path

class CleanerManager:
    """Handles system cleanup logic, RAM optimization, and cache clearing."""

    def __init__(self, log_callback=None):
        self.log = log_callback if log_callback else print
        self.prefs_file = Path.home() / ".config" / "zanemegapp" / "prefs.json"
        
        # Modules translated from the monolithic version
        self.modules = {
            "🧠 Smart RAM Optimizer": {"method": self._run_smart_optimizer, "sudo": False},
            "🧠 RAM Kernel Cache Flush (Sudo)": {"cmd": "sync && echo 3 > /proc/sys/vm/drop_caches && sysctl -w vm.compact_memory=1 && echo '[RAM] Kernel page cache flushed.'", "sudo": True},
            "⚙️ Background Service Optimizer (Sudo)": {"method": self._run_service_optimizer, "sudo": True},
            "Swap Memory Reset (Sudo)": {"cmd": "swapoff -a && swapon -a", "sudo": True},
            "APT Package Cache (Sudo)": {"cmd": "apt-get clean && apt-get autoclean && apt-get autoremove -y", "sudo": True},
            "Python Pip Cache": {"cmd": "pip cache purge", "sudo": False},
            "Systemd Journal Logs (Sudo)": {"cmd": "journalctl --vacuum-size=100M", "sudo": True},
            "User Trash Bin": {"cmd": "rm -rf ~/.local/share/Trash/files/* ~/.local/share/Trash/info/*", "sudo": False},
            "Thumbnail Cache": {"cmd": "rm -rf ~/.cache/thumbnails/* ~/.thumbnails/*", "sudo": False},
            "Flatpak Unused (Sudo)": {"cmd": "flatpak uninstall --unused -y", "sudo": True},
            "Docker Prune (Sudo)": {"cmd": "docker container prune -f && docker image prune -f", "sudo": True},
            "/tmp Old Files": {"cmd": "find /tmp -type f -atime +1 -delete 2>/dev/null; true", "sudo": False},
        }

    def load_prefs(self):
        try:
            if self.prefs_file.exists():
                with open(self.prefs_file, "r") as f:
                    return json.load(f)
        except Exception:
            pass
        return {}

    def save_prefs(self, prefs):
        try:
            self.prefs_file.parent.mkdir(parents=True, exist_ok=True)
            with open(self.prefs_file, "w") as f:
                json.dump(prefs, f)
        except Exception as e:
            self.log(f"[ERR] Saving prefs: {e}")

    def execute_cleanup(self, selected_modules, sudo_password=None):
        self.log("="*60)
        self.log("STARTING SYSTEM CLEANUP PROTOCOL")
        self.log("="*60)

        for name in selected_modules:
            if name not in self.modules:
                continue
                
            info = self.modules[name]
            self.log(f"[*] Executing: {name}...")
            
            try:
                if "method" in info:
                    # Run a python method directly
                    info["method"]()
                elif "cmd" in info:
                    # Run a shell command
                    cmd = f"sudo -S bash -c \"{info['cmd']}\"" if info["sudo"] else f"bash -c \"{info['cmd']}\""
                    p = subprocess.Popen(
                        cmd, shell=True, stdin=subprocess.PIPE, 
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
                    )
                    out, _ = p.communicate(input=sudo_password + '\n' if info["sudo"] and sudo_password else None)
                    if p.returncode == 0:
                        self.log(f"[✓] {name} completed successfully.")
                    else:
                        cleaned = out.replace('[sudo] password for zane: ', '').strip()
                        self.log(f"[!] {name} output: {cleaned}")
            except Exception as e:
                self.log(f"[ERR] Failed {name}: {str(e)}")

        self.log("="*60)
        self.log("CLEANUP PROTOCOL FINISHED")
        self.log("="*60)

    def _run_smart_optimizer(self):
        # 1. Kill bloat
        BLOAT_PATTERNS = [
            'tracker-miner', 'tracker-extract', 'tracker-store',
            'evolution-data', 'goa-daemon', 'baloo_file', 'xdg-desktop-portal',
        ]
        me = os.getpid()
        killed = []
        for proc in psutil.process_iter(['pid', 'name', 'username', 'memory_info']):
            try:
                if proc.pid == me: continue
                pname = proc.info['name'] or ''
                if proc.info['username'] != os.environ.get('USER', 'zane'): continue
                if any(pat in pname for pat in BLOAT_PATTERNS):
                    os.kill(proc.pid, signal.SIGTERM)
                    killed.append(pname)
            except:
                pass
        if killed:
            self.log(f"[RAM] Terminated {len(killed)} background bloat processes.")
        
        # 2. Zombies
        zombies = 0
        for proc in psutil.process_iter(['pid', 'status']):
            try:
                if proc.info['status'] == psutil.STATUS_ZOMBIE:
                    os.kill(proc.info['pid'], signal.SIGKILL)
                    zombies += 1
            except:
                pass
        if zombies:
            self.log(f"[RAM] Cleaned {zombies} zombie processes.")

    def _run_service_optimizer(self):
        # Uses pkexec or relies on the sudo loop wrapper
        SERVICES = ["cups.service", "ModemManager.service", "avahi-daemon.service"]
        stopped = 0
        for svc in SERVICES:
            r = subprocess.run(["systemctl", "is-active", "--quiet", svc])
            if r.returncode == 0:
                subprocess.run(["pkexec", "systemctl", "stop", svc])
                stopped += 1
        self.log(f"[SVC] Stopped {stopped} non-essential system services.")
