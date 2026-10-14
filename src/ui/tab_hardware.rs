use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::collections::VecDeque;

use crate::core::hardware::{HardwareManager, HardwareTelemetry};
use crate::theme::*;

pub struct TabHardware {
    pub telemetry: HardwareTelemetry,
    pub cpu_history: VecDeque<[f64; 2]>,
    pub gpu_history: VecDeque<[f64; 2]>,
    pub cooler_boost: bool,
    pub smart_boost: bool,
    pub battery_limit: u32,
    pub step_counter: usize,
    pub max_history: usize,
}

impl TabHardware {
    pub fn new() -> Self {
        Self {
            telemetry: HardwareTelemetry::default(),
            cpu_history: VecDeque::with_capacity(60),
            gpu_history: VecDeque::with_capacity(60),
            cooler_boost: false,
            smart_boost: false,
            battery_limit: 80,
            step_counter: 0,
            max_history: 60,
        }
    }

    pub fn update_telemetry(&mut self) {
        self.telemetry = HardwareManager::read_telemetry();
        self.step_counter += 1;

        let x = self.step_counter as f64;
        let cpu_y = self.telemetry.cpu_temp as f64;
        let gpu_y = self.telemetry.gpu_temp as f64;

        if self.cpu_history.len() >= self.max_history {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back([x, cpu_y]);

        if self.gpu_history.len() >= self.max_history {
            self.gpu_history.pop_front();
        }
        self.gpu_history.push_back([x, gpu_y]);

        // Auto trigger smart boost if enabled and temp exceeds 80°C
        if self.smart_boost && (self.telemetry.cpu_temp >= 80.0 || self.telemetry.gpu_temp >= 80.0) && !self.cooler_boost {
            self.cooler_boost = true;
            let _ = HardwareManager::set_cooler_boost(true);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        ui.add_space(6.0);

        // 1. Live Telemetry Card
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("REALTIME THERMAL TELEMETRY").strong().size(13.0).color(TEXT_MAIN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gpu_text = format!("GPU: {:.0}°C | {:.0}%", self.telemetry.gpu_temp, self.telemetry.gpu_fan_speed);
                        ui.label(egui::RichText::new(gpu_text).strong().size(12.0).color(RED_BRIGHT));
                        ui.add_space(8.0);
                        let cpu_text = format!("CPU: {:.0}°C | {:.0}%", self.telemetry.cpu_temp, self.telemetry.cpu_fan_speed);
                        ui.label(egui::RichText::new(cpu_text).strong().size(12.0).color(RED_PRIMARY));
                    });
                });

                ui.add_space(6.0);

                let cpu_pts: Vec<[f64; 2]> = self.cpu_history.iter().copied().collect();
                let gpu_pts: Vec<[f64; 2]> = self.gpu_history.iter().copied().collect();

                let plot = Plot::new("thermal_plot")
                    .legend(Legend::default())
                    .height(180.0)
                    .include_y(20.0)
                    .include_y(100.0)
                    .show_axes([false, true])
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new("CPU Temp (°C)", PlotPoints::new(cpu_pts))
                            .color(RED_BRIGHT)
                            .width(2.0),
                    );
                    plot_ui.line(
                        Line::new("GPU Temp (°C)", PlotPoints::new(gpu_pts))
                            .color(egui::Color32::from_rgb(255, 140, 140))
                            .width(1.8),
                    );
                });
            });

        ui.add_space(8.0);

        // 2. Power Profiles
        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(egui::Stroke::new(1.0, BORDER_DARK))
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("POWER PROFILES").strong().size(13.0).color(TEXT_MAIN));
                ui.add_space(6.0);

                ui.columns(3, |cols| {
                    if cols[0].button(egui::RichText::new("🔥 EXTREME\n144Hz | Max Fans | Full Power").strong()).clicked() {
                        match HardwareManager::apply_power_profile("extreme") {
                            Ok(msg) => set_status(msg, GREEN_OK),
                            Err(e) => set_status(e, RED_BRIGHT),
                        }
                    }
                    if cols[1].button(egui::RichText::new("⚖️ BALANCE\n48Hz | Smart Fans | Daily Use").strong()).clicked() {
                        match HardwareManager::apply_power_profile("balance") {
                            Ok(msg) => set_status(msg, GREEN_OK),
                            Err(e) => set_status(e, RED_BRIGHT),
                        }
                    }
                    if cols[2].button(egui::RichText::new("🔋 SUPER BATTERY\n48Hz | Silent | Bluetooth Off").strong()).clicked() {
                        match HardwareManager::apply_power_profile("battery") {
                            Ok(msg) => set_status(msg, GREEN_OK),
                            Err(e) => set_status(e, RED_BRIGHT),
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // 3. Fan Controls & Battery Health
        ui.columns(2, |cols| {
            // Fan Control Card
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(&mut cols[0], |ui| {
                    ui.label(egui::RichText::new("FAN CONTROLS & THERMAL MANAGEMENT").strong().size(12.0).color(TEXT_MAIN));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let boost_label = if self.cooler_boost { "❄️ COOLER BOOST [ON]" } else { "❄️ COOLER BOOST [OFF]" };
                        let btn_color = if self.cooler_boost { RED_PRIMARY } else { BG_CARD };
                        if ui.add(egui::Button::new(egui::RichText::new(boost_label).strong()).fill(btn_color)).clicked() {
                            self.cooler_boost = !self.cooler_boost;
                            match HardwareManager::set_cooler_boost(self.cooler_boost) {
                                Ok(_) => set_status(format!("Cooler Boost set to {}", if self.cooler_boost { "ON" } else { "OFF" }), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }

                        ui.checkbox(&mut self.smart_boost, egui::RichText::new("🤖 Smart Boost (>80°C)").size(11.0));
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Apply GF65 Custom Curve").clicked() {
                            match HardwareManager::apply_custom_fan_curve() {
                                Ok(_) => set_status("Applied ISW custom fan curve (16G41)".into(), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                        if ui.button("Reset to BIOS Auto").clicked() {
                            match HardwareManager::reset_fans_auto() {
                                Ok(_) => set_status("Fans reset to BIOS auto".into(), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                    });
                });

            // Battery Health Card
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(&mut cols[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("BATTERY HEALTH LIMIT").strong().size(12.0).color(TEXT_MAIN));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let badge = format!("CAP: {}%", self.battery_limit);
                            ui.label(egui::RichText::new(badge).strong().color(RED_BRIGHT));
                        });
                    });
                    ui.label(egui::RichText::new("Caps AC charging threshold to preserve Li-ion cells:").size(10.0).color(TEXT_MUTED));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if ui.button("60%\nMax Life").clicked() {
                            self.battery_limit = 60;
                            match HardwareManager::set_battery_limit(60) {
                                Ok(_) => set_status("Battery limit set to 60%".into(), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                        if ui.button("80%\nBalanced").clicked() {
                            self.battery_limit = 80;
                            match HardwareManager::set_battery_limit(80) {
                                Ok(_) => set_status("Battery limit set to 80%".into(), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                        if ui.button("100%\nFull Trip").clicked() {
                            self.battery_limit = 100;
                            match HardwareManager::set_battery_limit(100) {
                                Ok(_) => set_status("Battery limit set to 100%".into(), GREEN_OK),
                                Err(e) => set_status(e, RED_BRIGHT),
                            }
                        }
                    });
                });
        });
    }
}
