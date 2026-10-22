use eframe::egui;
use std::process::Child;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::core::idevice::IDeviceManager;
use crate::theme::*;

pub struct TabIDevice {
    pub logs: Vec<String>,
    pub syslog_proc: Option<Child>,
    pub iproxy_proc: Option<Child>,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabIDevice {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            logs: vec![
                "[*] iOS USB Device Toolkit ready.".into(),
                "[*] Connect an iPhone or iPad via USB cable, then click any diagnostic card on the left.".into(),
            ],
            syslog_proc: None,
            iproxy_proc: None,
            tx,
            rx,
        }
    }

    pub fn poll_logs(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            self.logs.push(line);
            if self.logs.len() > 1000 {
                self.logs.remove(0);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        self.poll_logs();

        legend_strip(
            ui,
            "USB diagnostics, live system logs, pairing, and local TCP port forwarding for iOS devices.",
            &[("Stream Active", GREEN_OK), ("USB Standby", CYAN_INFO)],
        );

        ui.add_space(6.0);

        let is_syslog = self.syslog_proc.is_some();
        let is_iproxy = self.iproxy_proc.is_some();

        let badge_text = if is_syslog && is_iproxy {
            "SYSLOG + TUNNEL ACTIVE"
        } else if is_syslog {
            "SYSLOG STREAMING"
        } else if is_iproxy {
            "SSH TUNNEL ACTIVE"
        } else {
            "USB READY"
        };

        let badge = if is_syslog || is_iproxy {
            (badge_text, GREEN_OK, GREEN_SUBTLE)
        } else {
            (badge_text, CYAN_INFO, CYAN_SUBTLE)
        };

        // Left action pane beside the console pane. The panes stack when 355 + 320 px cannot both
        // fit; neither one is ever squeezed to nothing.
        split(ui, 355.0, 320.0, |pane, index| {
            if index == 0 {
                cyber_card(
                    pane,
                    "📱 iOS USB DIAGNOSTIC TOOLS",
                    Some("Click a tool to inspect or connect to your device"),
                    RED_PRIMARY,
                    Some(badge),
                    |ui| {
                        if action_card_button(
                            ui,
                            "📱",
                            "Read Device Hardware Info",
                            "Displays model, iOS version, serial, and battery details",
                            Some(("QUERY", CYAN_INFO)),
                            CYAN_INFO,
                            false,
                        ) {
                            IDeviceManager::run_command(
                                &["ideviceinfo"],
                                "Device Info",
                                self.tx.clone(),
                            );
                            set_status("Reading iOS device info via USB...".into(), CYAN_INFO);
                        }

                        ui.add_space(5.0);

                        let syslog_title = if is_syslog {
                            "Stop Live Console Log Stream"
                        } else {
                            "Stream Live iOS System Logs"
                        };
                        let syslog_tag = if is_syslog {
                            ("STREAMING", GREEN_OK)
                        } else {
                            ("START", TEXT_MUTED)
                        };
                        if action_card_button(
                            ui,
                            if is_syslog { "🛑" } else { "📜" },
                            syslog_title,
                            "Streams real-time system & app debug messages over USB",
                            Some(syslog_tag),
                            if is_syslog { GREEN_OK } else { RED_PRIMARY },
                            is_syslog,
                        ) {
                            if let Some(mut proc) = self.syslog_proc.take() {
                                let _ = proc.kill();
                                self.tx
                                    .send("[*] idevicesyslog streaming stopped.".into())
                                    .ok();
                                set_status("Stopped iOS syslog stream".into(), TEXT_MUTED);
                            } else {
                                match IDeviceManager::start_syslog(self.tx.clone()) {
                                    Ok(proc) => {
                                        self.syslog_proc = Some(proc);
                                        set_status("Streaming live iOS syslog...".into(), GREEN_OK);
                                    }
                                    Err(e) => set_status(e, RED_BRIGHT),
                                }
                            }
                        }

                        ui.add_space(5.0);

                        let iproxy_title = if is_iproxy {
                            "Stop USB Port Tunnel (2222 -> 22)"
                        } else {
                            "Start USB SSH Port Tunnel (2222 -> 22)"
                        };
                        let iproxy_tag = if is_iproxy {
                            ("TUNNEL ON", GREEN_OK)
                        } else {
                            ("FORWARD", TEXT_MUTED)
                        };
                        if action_card_button(
                            ui,
                            if is_iproxy { "🛑" } else { "🔌" },
                            iproxy_title,
                            "Forwards localhost:2222 to port 22 on the iOS device",
                            Some(iproxy_tag),
                            if is_iproxy { GREEN_OK } else { AMBER_WARN },
                            is_iproxy,
                        ) {
                            if let Some(mut proc) = self.iproxy_proc.take() {
                                let _ = proc.kill();
                                self.tx.send("[*] iproxy tunnel stopped.".into()).ok();
                                set_status("Stopped iproxy USB tunnel".into(), TEXT_MUTED);
                            } else {
                                match IDeviceManager::start_iproxy(2222, 22, self.tx.clone()) {
                                    Ok(proc) => {
                                        self.iproxy_proc = Some(proc);
                                        set_status(
                                            "USB tunnel active on 127.0.0.1:2222 -> iOS :22".into(),
                                            GREEN_OK,
                                        );
                                    }
                                    Err(e) => set_status(e, RED_BRIGHT),
                                }
                            }
                        }

                        ui.add_space(5.0);

                        if action_card_button(
                            ui,
                            "🔑",
                            "Pair USB Device (Trust Computer)",
                            "Initiates the 'Trust This Computer' prompt on the iOS screen",
                            None,
                            GREEN_OK,
                            false,
                        ) {
                            IDeviceManager::run_command(
                                &["idevicepair", "pair"],
                                "Pair Device",
                                self.tx.clone(),
                            );
                        }

                        ui.add_space(5.0);

                        if action_card_button(
                            ui,
                            "✅",
                            "Validate Existing USB Trust",
                            "Verifies whether this computer is already trusted by the device",
                            None,
                            CYAN_INFO,
                            false,
                        ) {
                            IDeviceManager::run_command(
                                &["idevicepair", "validate"],
                                "Validate Pairing",
                                self.tx.clone(),
                            );
                        }
                    },
                );
            } else {
                cyber_console(
                    pane,
                    "idevice_console_scroll",
                    "iOS DEVICE // DIAGNOSTIC & CONSOLE OUTPUT",
                    &mut self.logs,
                    415.0,
                );
            }
        });
    }
}
