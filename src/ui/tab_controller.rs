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
        let init_target = format!(
            "[*] Controller Space Flight Deck initialized.\n[*] Target Directory: {}\n[*] Target Launcher: {}\n[*] Ready to launch Starship Cockpit.",
            manager.dir.display(),
            manager.script.display()
        );
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

        if let Some(pid) = self.running_pid {
            if !ControllerManager::is_running(Some(pid)) {
                self.running_pid = None;
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        self.poll_logs();
        ui.add_space(4.0);

        let is_running = ControllerManager::is_running(self.running_pid);
        let badge_str = if let Some(pid) = self.running_pid {
            format!("● ONLINE (PID {pid})")
        } else {
            "● OFFLINE / STANDBY".to_string()
        };
        let badge = if is_running {
            (badge_str.as_str(), GREEN_OK, GREEN_SUBTLE)
        } else {
            (badge_str.as_str(), TEXT_MUTED, BG_CARD)
        };

        // 1. Flight Deck Hero & Lifecycle Matrix Card
        cyber_card(
            ui,
            "🛸 CONTROLLER SPACE // STARSHIP FLIGHT DECK",
            Some("DUALSHOCK 4 & GAMEPAD TELEMETRY MATRIX"),
            RED_PRIMARY,
            Some(badge),
            |ui| {
                ui.label(
                    egui::RichText::new(
                        "1000Hz Polling Rate • Capacitive Touchpad Matrix • Gyro/Accel IMU • Haptics & RGB Lightbar Control",
                    )
                    .size(11.0)
                    .color(TEXT_MUTED),
                );
                ui.label(
                    egui::RichText::new(format!("Launcher Script: {}", self.manager.script.display()))
                        .monospace()
                        .size(10.0)
                        .color(CYAN_INFO),
                );

                ui.add_space(8.0);

                // Primary Lifecycle Buttons Row
                ui.horizontal(|ui| {
                    let start_btn = egui::Button::new(
                        egui::RichText::new("🚀 LAUNCH STARSHIP COCKPIT (start.sh)")
                            .strong()
                            .size(12.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(if is_running { BG_ELEVATED } else { RED_PRIMARY })
                    .min_size(egui::vec2(290.0, 38.0));

                    if ui.add(start_btn).clicked() {
                        if is_running {
                            set_status("Controller Space is already running".into(), AMBER_WARN);
                        } else {
                            match self.manager.launch(self.tx.clone()) {
                                Ok(pid) => {
                                    self.running_pid = Some(pid);
                                    set_status(format!("Controller Space launched (PID {pid})"), GREEN_OK);
                                }
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                    }

                    let stop_btn = egui::Button::new(egui::RichText::new("🛑 TERMINATE").strong().size(11.5))
                        .fill(if is_running { RED_CRIMSON } else { BG_CARD })
                        .min_size(egui::vec2(120.0, 38.0));

                    if ui.add_enabled(is_running, stop_btn).clicked() {
                        if let Some(pid) = self.running_pid {
                            ControllerManager::stop(pid, &self.tx);
                            self.running_pid = None;
                            set_status("Stopped Controller Space".into(), GREEN_OK);
                        }
                    }

                    let restart_btn = egui::Button::new(egui::RichText::new("🔄 RESTART").strong().size(11.5))
                        .fill(BG_CARD)
                        .min_size(egui::vec2(120.0, 38.0));

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

                ui.add_space(6.0);

                // Quick Subsystem Shortcuts
                ui.columns(3, |cols| {
                    if cols[0]
                        .add_sized(
                            [cols[0].available_width(), 32.0],
                            egui::Button::new(egui::RichText::new("💡 Launch LED & Rumble Matrix (led.py)").size(11.0)),
                        )
                        .clicked()
                    {
                        self.manager.launch_led(&self.tx);
                        set_status("Launched LED & Rumble Matrix Window".into(), CYAN_INFO);
                    }

                    if cols[1]
                        .add_sized(
                            [cols[1].available_width(), 32.0],
                            egui::Button::new(egui::RichText::new("📂 Open Project Directory").size(11.0)),
                        )
                        .clicked()
                    {
                        self.manager.open_folder();
                    }

                    if cols[2]
                        .add_sized(
                            [cols[2].available_width(), 32.0],
                            egui::Button::new(egui::RichText::new("🌐 GitHub: ZaneGoat/Controler-space").size(11.0)),
                        )
                        .clicked()
                    {
                        ControllerManager::open_github();
                    }
                });
            },
        );

        ui.add_space(8.0);

        // 2. Live Subprocess Telemetry Console
        cyber_console(
            ui,
            "controller_console_scroll",
            "CONTROLLER SPACE // SUBPROCESS STDOUT & STDERR STREAM",
            &mut self.logs,
            310.0,
        );
    }
}
