use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::core::headset::{HeadsetCommand, HeadsetManager, DEFAULT_MAC};
use crate::theme::*;

pub struct TabHeadset {
    pub mac_address: String,
    pub logs: Vec<String>,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabHeadset {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            mac_address: DEFAULT_MAC.to_string(),
            logs: vec![
                "[*] Headset Tab initialized.".into(),
                "[!] UNVERIFIED PROTOCOL: Real vendor payloads require Android HCI snoop extraction.".into(),
            ],
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

        // 1. Hero Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎧 SOUNDPEATS Q3 PRO // RFCOMM CONTROLLER").strong().size(13.0).color(RED_PRIMARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("MAC:").size(11.0).color(TEXT_MUTED));
                        ui.text_edit_singleline(&mut self.mac_address);
                    });
                });
            });

        ui.add_space(8.0);

        // 2. UNVERIFIED Banner
        egui::Frame::new()
            .fill(RED_SUBTLE)
            .stroke(egui::Stroke::new(1.0, RED_PRIMARY))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("⚠️ PROTOCOL NOTICE: UNVERIFIED PLACEHOLDER PAYLOADS").strong().size(12.0).color(RED_BRIGHT));
                ui.label(egui::RichText::new("The hex bytes for the Q3 Pro in ~/soundpeats_controller.py are unverified guesses and do NOT trigger ANC hardware states yet. Capture an Android Bluetooth HCI Snoop Log from the vendor app to discover the true protocol packets.").size(10.5).color(TEXT_MAIN));
            });

        ui.add_space(8.0);

        // 3. Control Buttons Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("HARDWARE MODE DISPATCH").strong().size(12.0).color(TEXT_MAIN));
                ui.add_space(6.0);

                let commands = [
                    HeadsetCommand::NoiseCancellation,
                    HeadsetCommand::Ambient,
                    HeadsetCommand::Normal,
                    HeadsetCommand::DisableControls,
                    HeadsetCommand::MovieMode,
                ];

                for cmd in commands {
                    let btn = egui::Button::new(egui::RichText::new(cmd.label()).strong().size(11.5))
                        .fill(BG_CARD)
                        .min_size(egui::vec2(ui.available_width(), 34.0));
                    if ui.add(btn).clicked() {
                        HeadsetManager::send_command(&self.mac_address, cmd, self.tx.clone());
                        set_status(format!("Sent {} (UNVERIFIED)", cmd.as_str()), RED_BRIGHT);
                    }
                    ui.add_space(3.0);
                }
            });

        ui.add_space(8.0);

        // 4. Output Log Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("RFCOMM CONSOLE & LOG").strong().size(11.0).color(TEXT_MAIN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear Log").clicked() {
                            self.logs.clear();
                        }
                    });
                });

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for line in &self.logs {
                            let color = if line.contains("UNVERIFIED") || line.contains("WARNING") {
                                RED_BRIGHT
                            } else if line.starts_with("[*]") {
                                egui::Color32::from_rgb(100, 180, 255)
                            } else {
                                egui::Color32::from_rgb(220, 220, 220)
                            };
                            ui.label(egui::RichText::new(line).monospace().size(10.5).color(color));
                        }
                    });
            });
    }
}
