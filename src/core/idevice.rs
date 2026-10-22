use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub struct IDeviceManager;

impl IDeviceManager {
    pub fn run_command(cmd: &[&str], title: &str, tx: Sender<String>) {
        let title_s = title.to_string();
        let cmd_vec: Vec<String> = cmd.iter().map(|s| s.to_string()).collect();

        tx.send(format!("[*] Running {title_s}...")).ok();
        tx.send("----------------------------------------".into())
            .ok();

        thread::spawn(move || {
            let prog = match cmd_vec.first() {
                Some(p) => p,
                None => return,
            };
            let args = &cmd_vec[1..];

            let out = Command::new(prog).args(args).output();

            match out {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if !stdout.is_empty() {
                        for line in stdout.lines() {
                            tx.send(line.to_string()).ok();
                        }
                    }
                    if !stderr.is_empty() {
                        tx.send(format!("[!] STDERR:\n{stderr}")).ok();
                    }
                    if o.status.success() && stdout.is_empty() && stderr.is_empty() {
                        tx.send("[✓] Command completed successfully with no output.".into())
                            .ok();
                    }
                }
                Err(e) => {
                    tx.send(format!("[ERR] Failed to execute {prog}: {e}")).ok();
                }
            }
        });
    }

    pub fn start_syslog(tx: Sender<String>) -> Result<Child, String> {
        let mut child = Command::new("idevicesyslog")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn idevicesyslog: {e}"))?;

        tx.send("[*] idevicesyslog started. Streaming real-time iOS console...".into())
            .ok();

        if let Some(stdout) = child.stdout.take() {
            let tx_out = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tx_out.send(line).ok();
                }
            });
        }

        Ok(child)
    }

    pub fn start_iproxy(
        local_port: u16,
        device_port: u16,
        tx: Sender<String>,
    ) -> Result<Child, String> {
        let mut child = Command::new("iproxy")
            .args([local_port.to_string(), device_port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn iproxy: {e}"))?;

        tx.send(format!(
            "[*] iproxy forward active: 127.0.0.1:{local_port} -> iOS :{device_port}"
        ))
        .ok();

        if let Some(stdout) = child.stdout.take() {
            let tx_out = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tx_out.send(line).ok();
                }
            });
        }

        Ok(child)
    }
}
