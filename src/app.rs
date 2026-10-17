use eframe::egui;
use std::time::{Duration, Instant};

use crate::theme::*;
use crate::ui::tab_cleaner::TabCleaner;
use crate::ui::tab_controller::TabController;
use crate::ui::tab_hardware::TabHardware;
use crate::ui::tab_headset::TabHeadset;
use crate::ui::tab_idevice::TabIDevice;
use crate::ui::tab_security::TabSecurity;
use crate::ui::AppTab;

pub struct ZaneMegApp {
    pub current_tab: AppTab,
    pub tab_hardware: TabHardware,
    pub tab_controller: TabController,
    pub tab_cleaner: TabCleaner,
    pub tab_headset: TabHeadset,
    pub tab_security: TabSecurity,
    pub tab_idevice: TabIDevice,
    pub status_message: String,
    pub status_color: egui::Color32,
    pub last_telemetry_tick: Instant,
}

impl ZaneMegApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let mut app = Self {
            current_tab: AppTab::Hardware,
            tab_hardware: TabHardware::new(),
            tab_controller: TabController::new(),
            tab_cleaner: TabCleaner::new(),
            tab_headset: TabHeadset::new(),
            tab_security: TabSecurity::new(),
            tab_idevice: TabIDevice::new(),
            status_message: "ZaneMegApp Core Matrix Online".into(),
            status_color: GREEN_OK,
            last_telemetry_tick: Instant::now(),
        };

        app.tab_hardware.update_telemetry();
        app.tab_cleaner.update_resources();
        app
    }

    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.status_message = msg;
        self.status_color = color;
    }
}

impl eframe::App for ZaneMegApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Telemetry update interval (every 1 second)
        if self.last_telemetry_tick.elapsed() >= Duration::from_millis(1000) {
            self.last_telemetry_tick = Instant::now();
            self.tab_hardware.update_telemetry();
            self.tab_cleaner.update_resources();
            self.tab_security.refresh_state();
        }

        // Request repaint every 500ms to keep animations and graphs fluid
        ui.ctx().request_repaint_after(Duration::from_millis(500));

        // ═══════════════════════════════════════════════════════════════════
        //  1. TOP CYBER-DECK TELEMETRY HUD HEADER
        // ═══════════════════════════════════════════════════════════════════
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(14, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Logo & Rig Identification
                    ui.label(
                        egui::RichText::new("⚡ ZANE MEGAPP")
                            .strong()
                            .size(16.5)
                            .color(RED_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("V2.5 CYBER-DECK // MSI GF65 THIN")
                            .monospace()
                            .size(9.5)
                            .color(TEXT_MUTED),
                    );

                    // Right-aligned Mini-HUD Telemetry Pills
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Shield Status Pill
                        let (s_text, s_fg, s_bg) = if self.tab_security.is_isolated {
                            ("🛡️ ISOLATED", RED_BRIGHT, RED_SUBTLE)
                        } else {
                            ("🛡️ OPEN", GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(ui, s_text, s_fg, s_bg);

                        // Battery Pill
                        let bat_text = format!("⚡ {}%", self.tab_hardware.telemetry.battery_pct);
                        status_pill(ui, &bat_text, TEXT_MAIN, BG_CARD);

                        // NVMe Disk % Pill
                        let disk_pct = self.tab_cleaner.resources.disk_percent;
                        let disk_text = format!("NVMe: {:.0}%", disk_pct);
                        status_pill(ui, &disk_text, CYAN_INFO, CYAN_SUBTLE);

                        // RAM % Pill
                        let ram_pct = self.tab_cleaner.resources.ram_percent;
                        let (ram_fg, ram_bg) = if ram_pct >= 85.0 {
                            (RED_BRIGHT, RED_SUBTLE)
                        } else if ram_pct >= 65.0 {
                            (AMBER_WARN, AMBER_SUBTLE)
                        } else {
                            (GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(ui, &format!("RAM: {:.0}%", ram_pct), ram_fg, ram_bg);

                        // CPU Temp & Load Pill
                        let cpu_t = self.tab_hardware.telemetry.cpu_temp;
                        let cpu_load = self.tab_hardware.telemetry.cpu_load_pct;
                        let (cpu_fg, cpu_bg) = if cpu_t >= 80.0 {
                            (RED_BRIGHT, RED_SUBTLE)
                        } else if cpu_t >= 65.0 {
                            (AMBER_WARN, AMBER_SUBTLE)
                        } else {
                            (GREEN_OK, GREEN_SUBTLE)
                        };
                        status_pill(
                            ui,
                            &format!("CPU: {:.0}°C ({:.0}%)", cpu_t, cpu_load),
                            cpu_fg,
                            cpu_bg,
                        );
                    });
                });
            });

        // Flight Deck Keyboard Shortcuts (F1 - F6)
        ui.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.current_tab = AppTab::Hardware;
            } else if i.key_pressed(egui::Key::F2) {
                self.current_tab = AppTab::Controller;
            } else if i.key_pressed(egui::Key::F3) {
                self.current_tab = AppTab::Cleaner;
            } else if i.key_pressed(egui::Key::F4) {
                self.current_tab = AppTab::Headset;
            } else if i.key_pressed(egui::Key::F5) {
                self.current_tab = AppTab::Security;
            } else if i.key_pressed(egui::Key::F6) {
                self.current_tab = AppTab::IDevice;
            }
        });

        // ═══════════════════════════════════════════════════════════════════
        //  2. CYBER NAVIGATION RAIL // SEGMENTED TAB SELECTOR
        // ═══════════════════════════════════════════════════════════════════
        egui::Frame::new()
            .fill(BG_VOID)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (AppTab::Hardware, "F1", "⚡ HARDWARE", None),
                        (
                            AppTab::Controller,
                            "F2",
                            "🎮 CONTROLLER",
                            self.tab_controller.running_pid.map(|p| format!("PID {p}")),
                        ),
                        (
                            AppTab::Cleaner,
                            "F3",
                            "🧹 CLEANER",
                            if self.tab_cleaner.is_running {
                                Some("BUSY".to_string())
                            } else {
                                None
                            },
                        ),
                        (AppTab::Headset, "F4", "🎧 HEADSET", None),
                        (
                            AppTab::Security,
                            "F5",
                            "🛡️ SECURITY & OPS",
                            if self.tab_security.is_isolated {
                                Some("LOCKED".to_string())
                            } else {
                                None
                            },
                        ),
                        (AppTab::IDevice, "F6", "📱 iDEVICE", None),
                    ];

                    for (tab, hk, label, tag) in tabs {
                        let is_active = self.current_tab == tab;
                        let bg_color = if is_active { RED_CRIMSON } else { BG_PANEL };
                        let text_color = if is_active { egui::Color32::WHITE } else { TEXT_MAIN };
                        let stroke = if is_active {
                            egui::Stroke::new(1.2, RED_BRIGHT)
                        } else {
                            egui::Stroke::new(1.0, BORDER_DARK)
                        };

                        let btn_content = if let Some(ref t) = tag {
                            format!("[{hk}] {label} ({t})")
                        } else {
                            format!("[{hk}] {label}")
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(btn_content)
                                .strong()
                                .size(11.0)
                                .color(text_color),
                        )
                        .fill(bg_color)
                        .stroke(stroke)
                        .min_size(egui::vec2(0.0, 30.0));

                        if ui.add(btn).clicked() {
                            self.current_tab = tab;
                        }
                    }
                });
            });

        ui.add_space(2.0);

        // ═══════════════════════════════════════════════════════════════════
        //  3. CENTRAL SCROLLABLE TAB VIEWPORT
        // ═══════════════════════════════════════════════════════════════════
        let mut status_change: Option<(String, egui::Color32)> = None;
        let mut status_cb = |msg: String, color: egui::Color32| {
            status_change = Some((msg, color));
        };

        egui::ScrollArea::vertical()
            .id_salt("main_viewport_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.current_tab {
                    AppTab::Hardware => self.tab_hardware.ui(ui, &mut status_cb),
                    AppTab::Controller => self.tab_controller.ui(ui, &mut status_cb),
                    AppTab::Cleaner => self.tab_cleaner.ui(ui, &mut status_cb),
                    AppTab::Headset => self.tab_headset.ui(ui, &mut status_cb),
                    AppTab::Security => self.tab_security.ui(ui, &mut status_cb),
                    AppTab::IDevice => self.tab_idevice.ui(ui, &mut status_cb),
                }
            });

        if let Some((msg, color)) = status_change {
            self.set_status(msg, color);
        }

        // ═══════════════════════════════════════════════════════════════════
        //  4. BOTTOM COMMAND HUD STATUS BAR
        // ═══════════════════════════════════════════════════════════════════
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(14, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").size(10.0).color(self.status_color));
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(11.0)
                            .color(TEXT_MAIN),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sys_info = format!(
                            "LOAD: {:.0}% | CPU: {:.0}°C | GPU: {:.0}°C | NVMe: {:.0}°C | {:.2} GHz | UP: {}",
                            self.tab_hardware.telemetry.cpu_load_pct,
                            self.tab_hardware.telemetry.cpu_temp,
                            self.tab_hardware.telemetry.gpu_temp,
                            self.tab_hardware.telemetry.nvme_temp,
                            self.tab_hardware.telemetry.cpu_freq_ghz,
                            self.tab_hardware.telemetry.uptime_formatted
                        );
                        ui.label(
                            egui::RichText::new(sys_info)
                                .monospace()
                                .size(10.0)
                                .color(TEXT_MUTED),
                        );
                    });
                });
            });
    }
}
