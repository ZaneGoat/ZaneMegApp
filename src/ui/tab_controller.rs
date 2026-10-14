use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::core::controller::ControllerManager;
use crate::theme::*;

pub struct TabController {
    pub manager: ControllerManager,
    pub running_pid: Option<u32>,
    pub logs: Vec<String>,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl TabController {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let manager = ControllerManager::new();
        let init_target = format!("[*] Controller Space initialized.\n[*] Target Directory: {}\n[*] Target Launcher: {}\n[*] Ready to launch Starship Cockpit.", manager.dir.display(), manager.script.display());
        Self {
            manager,
            running_pid: None,
            logs: vec![init_target],
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

        // Check if running PID has exited
        if let Some(pid) = self.running_pid {
            if !ControllerManager::is_running(Some(pid)) {
                self.running_pid = None;
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        self.poll_logs();
        ui.add_space(6.0);

        let is_running = ControllerManager::is_running(self.running_pid);

        // 1. Hero Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("🛸 CONTROLLER SPACE // STARSHIP FLIGHT DECK").strong().size(14.0).color(RED_PRIMARY));
                        ui.label(egui::RichText::new("PS4 DualShock 4 & Gamepad Telemetry • 1000Hz Polling Matrix • Capacitive Touchpad • Haptics & LEDs").size(10.0).color(TEXT_MUTED));
                        ui.label(egui::RichText::new(format!("Target: {}", self.manager.script.display())).monospace().size(9.0).color(egui::Color32::from_rgb(170, 102, 102)));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (badge_text, badge_color) = if is_running {
                            ("● ONLINE", GREEN_OK)
                        } else {
                            ("● OFFLINE", TEXT_MUTED)
                        };
                        ui.label(egui::RichText::new(badge_text).strong().size(12.0).color(badge_color));
                    });
                });
            });

        ui.add_space(8.0);

        // 2. Action Controls Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("HARDWARE MATRIX CONTROLS & SHORTCUTS").strong().size(12.0).color(TEXT_MAIN));
                ui.add_space(6.0);

                // Main Buttons Row
                ui.horizontal(|ui| {
                    let start_btn = egui::Button::new(egui::RichText::new("🚀 START CONTROLLER SPACE (start.sh)").strong().size(12.0))
                        .fill(RED_PRIMARY)
                        .min_size(egui::vec2(280.0, 38.0));
                    if ui.add(start_btn).clicked() {
                        if is_running {
                            set_status("Controller Space is already running".into(), RED_BRIGHT);
                        } else {
                            match self.manager.launch(self.tx.clone()) {
                                Ok(pid) => {
                                    self.running_pid = Some(pid);
                                    set_status(format!("Controller Space started (PID {pid})"), GREEN_OK);
                                }
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                    }

                    let stop_btn = egui::Button::new(egui::RichText::new("🛑 STOP").strong().size(12.0))
                        .fill(BG_CARD)
                        .min_size(egui::vec2(100.0, 38.0));
                    if ui.add(stop_btn).clicked() {
                        if let Some(pid) = self.running_pid {
                            ControllerManager::stop(pid, &self.tx);
                            self.running_pid = None;
                            set_status("Stopped Controller Space".into(), GREEN_OK);
                        } else {
                            set_status("No running instance detected".into(), TEXT_MUTED);
                        }
                    }

                    let restart_btn = egui::Button::new(egui::RichText::new("🔄 RESTART").strong().size(12.0))
                        .fill(BG_CARD)
                        .min_size(egui::vec2(100.0, 38.0));
                    if ui.add(restart_btn).clicked() {
                        if let Some(pid) = self.running_pid {
                            ControllerManager::stop(pid, &self.tx);
                        }
                        match self.manager.launch(self.tx.clone()) {
                            Ok(pid) => {
                                self.running_pid = Some(pid);
                                set_status(format!("Restarted Controller Space (PID {pid})"), GREEN_OK);
                            }
                            Err(e) => set_status(e, RED_BRIGHT),
                        }
                    }
                });

                ui.add_space(8.0);

                // Links & Subsystem Launchers Row
                ui.horizontal(|ui| {
                    if ui.button("🌐 GitHub: ZaneGoat/Controler-space").clicked() {
                        ControllerManager::open_github();
                    }
                    if ui.button("📂 Open Controler Space Folder").clicked() {
                        self.manager.open_folder();
                    }
                    if ui.button("💡 Launch LED & Rumble Window").clicked() {
                        self.manager.launch_led(&self.tx);
                    }
                });
            });

        ui.add_space(8.0);

        // 3. Terminal Output Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CONTROLLER SPACE // CONSOLE & RUNTIME TELEMETRY").strong().size(11.0).color(TEXT_MAIN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear Log").clicked() {
                            self.logs.clear();
                        }
                    });
                });

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(240.0)
                    .show(ui, |ui| {
                        for line in &self.logs {
                            ui.label(egui::RichText::new(line).monospace().size(10.5).color(egui::Color32::from_rgb(220, 220, 220)));
                        }
                    });
            });
    }
}
