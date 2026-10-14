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
            status_message: "Ready".into(),
            status_color: TEXT_MUTED,
            last_telemetry_tick: Instant::now(),
        };

        app.tab_hardware.update_telemetry();
        app
    }

    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.status_message = msg;
        self.status_color = color;
    }
}

impl eframe::App for ZaneMegApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Telemetry update interval (1 sec)
        if self.last_telemetry_tick.elapsed() >= Duration::from_millis(1000) {
            self.last_telemetry_tick = Instant::now();
            self.tab_hardware.update_telemetry();
            self.tab_security.refresh_state();
        }

        // Request a redraw every 500ms to keep animations and graphs smooth
        ui.ctx().request_repaint_after(Duration::from_millis(500));

        // 1. Top Header Panel
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("ZANE MEGAPP")
                            .strong()
                            .size(17.0)
                            .color(RED_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("V2.0 / RUST CORE MATRIX")
                            .size(10.0)
                            .color(TEXT_MUTED),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let tabs = [
                            AppTab::IDevice,
                            AppTab::Security,
                            AppTab::Headset,
                            AppTab::Cleaner,
                            AppTab::Controller,
                            AppTab::Hardware,
                        ];

                        for tab in tabs {
                            let is_active = self.current_tab == tab;
                            let btn_color = if is_active { RED_PRIMARY } else { BG_CARD };
                            let text_color = if is_active { egui::Color32::WHITE } else { TEXT_MAIN };

                            let btn = egui::Button::new(
                                egui::RichText::new(tab.title())
                                    .strong()
                                    .size(11.0)
                                    .color(text_color),
                            )
                            .fill(btn_color)
                            .min_size(egui::vec2(0.0, 28.0));

                            if ui.add(btn).clicked() {
                                self.current_tab = tab;
                            }
                        }
                    });
                });
            });

        ui.add_space(4.0);

        // 2. Central Tab Content Panel
        let mut status_change: Option<(String, egui::Color32)> = None;
        let mut status_cb = |msg: String, color: egui::Color32| {
            status_change = Some((msg, color));
        };

        egui::ScrollArea::vertical()
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

        // 3. Bottom Status Bar
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .inner_margin(egui::Margin::symmetric(14, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").size(10.0).color(self.status_color));
                    ui.label(egui::RichText::new(&self.status_message).size(11.0).color(TEXT_MAIN));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sys_info = format!("CPU: {:.0}°C | GPU: {:.0}°C", self.tab_hardware.telemetry.cpu_temp, self.tab_hardware.telemetry.gpu_temp);
                        ui.label(egui::RichText::new(sys_info).monospace().size(10.0).color(TEXT_MUTED));
                    });
                });
            });
    }
}
