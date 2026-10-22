use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub struct ControllerManager {
    pub dir: PathBuf,
    pub script: PathBuf,
}

impl ControllerManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/zane".into());
        let primary = Path::new(&home).join("Desktop").join("Controler space");
        let fallback = Path::new(&home).join("controller-dashboard");

        let dir = if primary.exists() { primary } else { fallback };
        let script = dir.join("start.sh");

        Self { dir, script }
    }

    #[allow(dead_code)]
    pub fn is_installed(&self) -> bool {
        self.script.exists()
    }

    pub fn launch(&self, tx: Sender<String>) -> Result<u32, String> {
        if !self.script.exists() {
            return Err(format!(
                "Controller Space start.sh not found at {}",
                self.script.display()
            ));
        }

        let mut child = Command::new("/bin/bash")
            .arg(&self.script)
            .current_dir(&self.dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn start.sh: {e}"))?;

        let pid = child.id();
        tx.send(format!("[*] Starship Cockpit launched with PID {pid}"))
            .ok();

        // Stream stdout in a thread
        if let Some(stdout) = child.stdout.take() {
            let tx_out = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tx_out.send(line).ok();
                }
            });
        }

        // Stream stderr in a thread
        if let Some(stderr) = child.stderr.take() {
            let tx_err = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    tx_err.send(format!("[ERR] {line}")).ok();
                }
            });
        }

        thread::spawn(move || {
            let _ = child.wait();
            tx.send("[*] Controller Space process exited.".into()).ok();
        });

        Ok(pid)
    }

    pub fn stop(pid: u32, tx: &Sender<String>) {
        tx.send(format!("[*] Stopping Controller Space PID {pid}..."))
            .ok();
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
        // Give it a brief moment then kill -9 if still up
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
        tx.send("[✓] Process terminated.".into()).ok();
    }

    pub fn open_folder(&self) {
        let _ = Command::new("xdg-open").arg(&self.dir).spawn();
    }

    pub fn open_github() {
        let _ = Command::new("xdg-open")
            .arg("https://github.com/ZaneGoat/Controler-space")
            .spawn();
    }

    pub fn launch_led(&self, tx: &Sender<String>) {
        let led_script = self.dir.join("led.py");
        if !led_script.exists() {
            tx.send(format!("[!] led.py not found at {}", led_script.display()))
                .ok();
            return;
        }

        tx.send("[*] Launching LED & Rumble Matrix Window...".into())
            .ok();
        let _ = Command::new("python3")
            .arg(&led_script)
            .current_dir(&self.dir)
            .spawn();
    }

    pub fn is_running(pid: Option<u32>) -> bool {
        if let Some(p) = pid {
            let status = Command::new("kill").args(["-0", &p.to_string()]).status();
            status.map(|s| s.success()).unwrap_or(false)
        } else {
            false
        }
    }
}
