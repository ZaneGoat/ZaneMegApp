use eframe::egui;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::cleaner::{CleanerManager, MODULES};
use crate::theme::*;

pub struct TabCleaner {
    pub selected_ids: HashSet<String>,
    pub logs: Vec<String>,
    pub is_running: bool,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabCleaner {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let mut selected_ids = HashSet::new();
        for m in MODULES {
            if m.default_checked {
                selected_ids.insert(m.id.to_string());
            }
        }
        Self {
            selected_ids,
            logs: vec!["[*] System Cleaner initialized. Select target cleanup modules and execute.".into()],
            is_running: false,
            tx,
            rx,
        }
    }

    pub fn poll_logs(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            if line.contains("CLEANUP PROTOCOL COMPLETED") {
                self.is_running = false;
            }
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
            // Left sidebar: modules & presets
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.label(egui::RichText::new("CLEANUP MODULES").strong().size(13.0).color(RED_PRIMARY));
                    ui.add_space(4.0);

                    // Preset buttons
                    ui.horizontal(|ui| {
                        if ui.button("All").clicked() {
                            for m in MODULES {
                                self.selected_ids.insert(m.id.to_string());
                            }
                        }
                        if ui.button("None").clicked() {
                            self.selected_ids.clear();
                        }
                        if ui.button("Safe Default").clicked() {
                            self.selected_ids.clear();
                            for m in MODULES {
                                if m.default_checked {
                                    self.selected_ids.insert(m.id.to_string());
                                }
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(380.0)
                        .show(ui, |ui| {
                            for m in MODULES {
                                let mut checked = self.selected_ids.contains(m.id);
                                let root_tag = if m.requires_root { " (root)" } else { "" };
                                let label_text = format!("{}{}", m.name, root_tag);

                                if ui.checkbox(&mut checked, egui::RichText::new(label_text).size(11.0))
                                    .on_hover_text(m.description)
                                    .changed()
                                {
                                    if checked {
                                        self.selected_ids.insert(m.id.to_string());
                                    } else {
                                        self.selected_ids.remove(m.id);
                                    }
                                }
                            }
                        });

                    ui.add_space(10.0);

                    let exec_btn = egui::Button::new(
                        egui::RichText::new(if self.is_running { "⏳ RUNNING..." } else { "🚀 EXECUTE CLEANUP" })
                            .strong()
                            .size(13.0),
                    )
                    .fill(if self.is_running { BG_CARD } else { RED_PRIMARY })
                    .min_size(egui::vec2(260.0, 42.0));

                    if ui.add_enabled(!self.is_running, exec_btn).clicked() {
                        self.is_running = true;
                        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            CleanerManager::execute_cleanup(&ids, tx);
                        });
                        set_status("Started system cleanup protocol".into(), GREEN_OK);
                    }
                });

            ui.add_space(8.0);

            // Right Pane: Console Output Log
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SYSTEM CLEANER // REALTIME EXECUTION CONSOLE").strong().size(12.0).color(TEXT_MAIN));
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
                                let color = if line.starts_with("[✓]") {
                                    GREEN_OK
                                } else if line.starts_with("[!]") || line.starts_with("[ERR]") {
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
        });
    }
}
