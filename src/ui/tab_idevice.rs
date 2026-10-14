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
            logs: vec!["[*] iDevice Toolkit ready. Connect your iOS device via USB.".into()],
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
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            // Left Action Panel
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.label(egui::RichText::new("iOS DEVICE TOOLS").strong().size(13.0).color(RED_PRIMARY));
                    ui.add_space(8.0);

                    if ui.button("📱 Device Info (ideviceinfo)").clicked() {
                        IDeviceManager::run_command(&["ideviceinfo"], "Device Info", self.tx.clone());
                        set_status("Querying ideviceinfo...".into(), GREEN_OK);
                    }

                    ui.add_space(4.0);

                    let syslog_label = if self.syslog_proc.is_some() { "🛑 Stop Syslog" } else { "📜 Stream Syslog (idevicesyslog)" };
                    if ui.button(syslog_label).clicked() {
                        if let Some(mut proc) = self.syslog_proc.take() {
                            let _ = proc.kill();
                            self.tx.send("[*] Syslog streaming stopped.".into()).ok();
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

                    let iproxy_label = if self.iproxy_proc.is_some() { "🛑 Stop SSH Proxy" } else { "🔌 Forward Port 2222 -> 22 (iproxy)" };
                    if ui.button(iproxy_label).clicked() {
                        if let Some(mut proc) = self.iproxy_proc.take() {
                            let _ = proc.kill();
                            self.tx.send("[*] iproxy tunnel stopped.".into()).ok();
                            set_status("Stopped iproxy".into(), TEXT_MUTED);
                        } else {
                            match IDeviceManager::start_iproxy(2222, 22, self.tx.clone()) {
                                Ok(proc) => {
                                    self.iproxy_proc = Some(proc);
                                    set_status("iproxy active on 127.0.0.1:2222".into(), GREEN_OK);
                                }
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                    }

                    ui.add_space(4.0);

                    if ui.button("🔑 Pair Device (idevicepair)").clicked() {
                        IDeviceManager::run_command(&["idevicepair", "pair"], "Pairing Device", self.tx.clone());
                    }

                    ui.add_space(4.0);

                    if ui.button("✅ Validate Pairing").clicked() {
                        IDeviceManager::run_command(&["idevicepair", "validate"], "Validating Pair", self.tx.clone());
                    }
                });

            ui.add_space(8.0);

            // Right Console Output
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("iDEVICE // CONSOLE & RUNTIME LOG").strong().size(11.5).color(TEXT_MAIN));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear Log").clicked() {
                                self.logs.clear();
                            }
                        });
                    });

                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(480.0)
                        .show(ui, |ui| {
                            for line in &self.logs {
                                ui.label(egui::RichText::new(line).monospace().size(10.5).color(egui::Color32::from_rgb(220, 220, 220)));
                            }
                        });
                });
        });
    }
}
