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
            "[*] Controller Space Flight Deck initialized.\n[*] Target Directory: {}\n[*] Target Launcher: {}\n[*] Click 'Launch Starship Cockpit' above to start 1000Hz gamepad telemetry.",
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

        legend_strip(
            ui,
            "Launch and manage the PS4 DualShock 4 / Gamepad 1000Hz dashboard, RGB lightbar, and haptics.",
            &[
                ("Cockpit Running", GREEN_OK),
                ("Standby / Ready", CYAN_INFO),
            ],
        );

        ui.add_space(6.0);

        let is_running = ControllerManager::is_running(self.running_pid);
        let badge_str = if let Some(pid) = self.running_pid {
            format!("● RUNNING (PID {pid})")
        } else {
            "● STANDBY / READY".to_string()
        };
        let badge = if is_running {
            (badge_str.as_str(), GREEN_OK, GREEN_SUBTLE)
        } else {
            (badge_str.as_str(), CYAN_INFO, CYAN_SUBTLE)
        };

        // 1. Flight Deck Main Controls Card
        cyber_card(
            ui,
            "🛸 CONTROLLER SPACE // STARSHIP FLIGHT DECK",
            Some("1000Hz Gamepad Polling, Touchpad, Gyro & Haptics"),
            RED_PRIMARY,
            Some(badge),
            |ui| {
                ui.columns(3, |cols| {
                    if action_card_button(
                        &mut cols[0],
                        "🚀",
                        "Launch Starship Cockpit",
                        "Starts start.sh dashboard & telemetry",
                        Some((if is_running { "ACTIVE" } else { "LAUNCH" }, GREEN_OK)),
                        RED_PRIMARY,
                        !is_running,
                    ) {
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

                    if action_card_button(
                        &mut cols[1],
                        "🔄",
                        "Restart Cockpit",
                        "Reloads the controller service cleanly",
                        Some(("RELOAD", CYAN_INFO)),
                        CYAN_INFO,
                        false,
                    ) {
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

                    if action_card_button(
                        &mut cols[2],
                        "🛑",
                        "Stop Cockpit",
                        "Closes the running controller service",
                        Some(("STOP", RED_BRIGHT)),
                        RED_BRIGHT,
                        is_running,
                    ) && is_running
                    {
                        if let Some(pid) = self.running_pid {
                            ControllerManager::stop(pid, &self.tx);
                            self.running_pid = None;
                            set_status("Stopped Controller Space".into(), GREEN_OK);
                        }
                    }
                });

                ui.add_space(6.0);

                // Quick Subsystem Shortcuts
                ui.columns(3, |cols| {
                    if action_card_button(
                        &mut cols[0],
                        "💡",
                        "LED & Rumble Matrix",
                        "Open standalone RGB lightbar & vibration tuner",
                        None,
                        AMBER_WARN,
                        false,
                    ) {
                        self.manager.launch_led(&self.tx);
                        set_status("Opened LED & Rumble Matrix Window".into(), CYAN_INFO);
                    }

                    if action_card_button(
                        &mut cols[1],
                        "📂",
                        "Open Project Folder",
                        "Browse Controller Space source files",
                        None,
                        CYAN_INFO,
                        false,
                    ) {
                        self.manager.open_folder();
                    }

                    if action_card_button(
                        &mut cols[2],
                        "🌐",
                        "GitHub Repository",
                        "Open ZaneGoat/Controler-space in browser",
                        None,
                        GREEN_OK,
                        false,
                    ) {
                        ControllerManager::open_github();
                    }
                });
            },
        );

        ui.add_space(6.0);

        // 2. Live Subprocess Telemetry Console
        cyber_console(
            ui,
            "controller_console_scroll",
            "CONTROLLER SPACE // LIVE OUTPUT LOG",
            &mut self.logs,
            320.0,
        );
    }
}
