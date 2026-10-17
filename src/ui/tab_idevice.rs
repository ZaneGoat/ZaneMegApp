use eframe::egui;
use std::process::Child;
use std::sync::mpsc::{channel, Receiver, Sender};

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
                "[*] iDevice Toolkit ready.".into(),
                "[*] Connect an iOS device via USB and pair to enable diagnostics.".into(),
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
        ui.add_space(4.0);

        let is_syslog = self.syslog_proc.is_some();
        let is_iproxy = self.iproxy_proc.is_some();

        let badge_text = if is_syslog && is_iproxy {
            "SYSLOG + IPROXY ACTIVE"
        } else if is_syslog {
            "SYSLOG STREAMING"
        } else if is_iproxy {
            "IPROXY TUNNEL ACTIVE"
        } else {
            "USB STANDBY"
        };

        let badge = if is_syslog || is_iproxy {
            (badge_text, GREEN_OK, GREEN_SUBTLE)
        } else {
            (badge_text, TEXT_MUTED, BG_CARD)
        };

        ui.horizontal_top(|ui| {
            // Left Action Sidebar
            ui.vertical(|ui| {
                ui.set_width(310.0);

                cyber_card(
                    ui,
                    "iOS USB TOOLKIT",
                    Some("LIBIMOBILEDEVICE"),
                    RED_PRIMARY,
                    Some(badge),
                    |ui| {
                        // Diagnostic tools
                        let info_btn = egui::Button::new(
                            egui::RichText::new("📱 Query Device Info (ideviceinfo)")
                                .strong()
                                .size(11.0),
                        )
                        .fill(BG_CARD)
                        .min_size(egui::vec2(ui.available_width(), 34.0));

                        if ui.add(info_btn).clicked() {
                            IDeviceManager::run_command(&["ideviceinfo"], "Device Info", self.tx.clone());
                            set_status("Querying ideviceinfo...".into(), CYAN_INFO);
                        }

                        ui.add_space(4.0);

                        // Syslog Toggle Button
                        let syslog_text = if is_syslog {
                            "🛑 STOP SYSLOG STREAM"
                        } else {
                            "📜 STREAM LIVE SYSLOG (idevicesyslog)"
                        };
                        let syslog_fill = if is_syslog { RED_CRIMSON } else { BG_CARD };
                        let syslog_btn = egui::Button::new(
                            egui::RichText::new(syslog_text)
                                .strong()
                                .size(11.0)
                                .color(if is_syslog { egui::Color32::WHITE } else { TEXT_MAIN }),
                        )
                        .fill(syslog_fill)
                        .min_size(egui::vec2(ui.available_width(), 34.0));

                        if ui.add(syslog_btn).clicked() {
                            if let Some(mut proc) = self.syslog_proc.take() {
                                let _ = proc.kill();
                                self.tx.send("[*] idevicesyslog streaming stopped.".into()).ok();
                                set_status("Stopped idevicesyslog".into(), TEXT_MUTED);
                            } else {
                                match IDeviceManager::start_syslog(self.tx.clone()) {
                                    Ok(proc) => {
                                        self.syslog_proc = Some(proc);
                                        set_status("Streaming idevicesyslog...".into(), GREEN_OK);
                                    }
                                    Err(e) => set_status(e, RED_BRIGHT),
                                }
                            }
                        }

                        ui.add_space(4.0);

                        // iProxy SSH Forwarding Toggle
                        let iproxy_text = if is_iproxy {
                            "🛑 STOP SSH PROXY TUNNEL"
                        } else {
                            "🔌 FORWARD 127.0.0.1:2222 -> :22"
                        };
                        let iproxy_fill = if is_iproxy { RED_CRIMSON } else { BG_CARD };
                        let iproxy_btn = egui::Button::new(
                            egui::RichText::new(iproxy_text)
                                .strong()
                                .size(11.0)
                                .color(if is_iproxy { egui::Color32::WHITE } else { TEXT_MAIN }),
                        )
                        .fill(iproxy_fill)
                        .min_size(egui::vec2(ui.available_width(), 34.0));

                        if ui.add(iproxy_btn).clicked() {
                            if let Some(mut proc) = self.iproxy_proc.take() {
                                let _ = proc.kill();
                                self.tx.send("[*] iproxy tunnel stopped.".into()).ok();
                                set_status("Stopped iproxy tunnel".into(), TEXT_MUTED);
                            } else {
                                match IDeviceManager::start_iproxy(2222, 22, self.tx.clone()) {
                                    Ok(proc) => {
                                        self.iproxy_proc = Some(proc);
                                        set_status("iproxy active: 127.0.0.1:2222 -> iOS :22".into(), GREEN_OK);
                                    }
                                    Err(e) => set_status(e, RED_BRIGHT),
                                }
                            }
                        }

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Pairing Controls
                        ui.columns(2, |cols| {
                            if cols[0]
                                .add_sized(
                                    [cols[0].available_width(), 30.0],
                                    egui::Button::new(egui::RichText::new("🔑 Pair Device").size(11.0)),
                                )
                                .clicked()
                            {
                                IDeviceManager::run_command(
                                    &["idevicepair", "pair"],
                                    "Pair Device",
                                    self.tx.clone(),
                                );
                            }

                            if cols[1]
                                .add_sized(
                                    [cols[1].available_width(), 30.0],
                                    egui::Button::new(egui::RichText::new("✅ Validate").size(11.0)),
                                )
                                .clicked()
                            {
                                IDeviceManager::run_command(
                                    &["idevicepair", "validate"],
                                    "Validate Pairing",
                                    self.tx.clone(),
                                );
                            }
                        });
                    },
                );
            });

            ui.add_space(6.0);

            // Right Console Output
            ui.vertical(|ui| {
                cyber_console(
                    ui,
                    "idevice_console_scroll",
                    "iDEVICE DIAGNOSTIC // CONSOLE & RUNTIME STREAM",
                    &mut self.logs,
                    415.0,
                );
            });
        });
    }
}
