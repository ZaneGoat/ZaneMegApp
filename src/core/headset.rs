use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub const DEFAULT_MAC: &str = "41:42:F0:AC:0F:EF";

// UNVERIFIED: Commands for SoundPEATS Q3 Pro. Payload bytes must be captured
// via Android HCI snoop logs from the official vendor app.
#[derive(Clone, Copy, Debug)]
pub enum HeadsetCommand {
    NoiseCancellation,
    Ambient,
    Normal,
    DisableControls,
    MovieMode,
}

impl HeadsetCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoiseCancellation => "noise_cancelation",
            Self::Ambient => "ambient",
            Self::Normal => "normal",
            Self::DisableControls => "disable_controls",
            Self::MovieMode => "movie_mode",
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::NoiseCancellation => "🛡️ ANC (Active Noise Cancellation) [UNVERIFIED]",
            Self::Ambient => "👂 Transparency / Ambient Sound [UNVERIFIED]",
            Self::Normal => "⚖️ Normal Mode (ANC Off) [UNVERIFIED]",
            Self::DisableControls => "🔒 Disable Touch Controls [UNVERIFIED]",
            Self::MovieMode => "🎬 Movie / Low Latency Mode [UNVERIFIED]",
        }
    }
}

pub struct HeadsetManager;

impl HeadsetManager {
    /// Dispatches command to ~/soundpeats_controller.py.
    /// NOTE: All payloads sent to this headset are currently UNVERIFIED placeholders.
    pub fn send_command(mac: &str, cmd: HeadsetCommand, tx: Sender<String>) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/zane".into());
        let script = Path::new(&home).join("soundpeats_controller.py");

        if !script.exists() {
            tx.send(format!("[ERR] soundpeats_controller.py not found at {}", script.display())).ok();
            return;
        }

        let cmd_str = cmd.as_str().to_string();
        let mac_str = mac.to_string();

        tx.send("------------------------------------------------------------".into()).ok();
        tx.send(format!("[*] Sending UNVERIFIED command: {cmd_str} to {mac_str}")).ok();
        tx.send("[!] NOTICE: The SoundPEATS Q3 Pro RFCOMM protocol requires reverse-engineering.".into()).ok();
        tx.send("[!] Capture an Android Bluetooth HCI snoop log to extract genuine vendor payloads.".into()).ok();
        tx.send("------------------------------------------------------------".into()).ok();

        thread::spawn(move || {
            let mut child = match Command::new("python3")
                .arg(&script)
                .arg(&mac_str)
                .arg(&cmd_str)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    tx.send(format!("[ERR] Failed to spawn controller: {e}")).ok();
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tx.send(line).ok();
                }
            }

            if let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    tx.send(format!("[ERR] {line}")).ok();
                }
            }

            let _ = child.wait();
        });
    }
}
