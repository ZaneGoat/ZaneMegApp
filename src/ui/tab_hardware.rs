use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::hardware::{HardwareManager, HardwareTelemetry};
use crate::theme::*;

pub struct TabHardware {
    pub telemetry: HardwareTelemetry,
    pub cpu_history: VecDeque<[f64; 2]>,
    pub gpu_history: VecDeque<[f64; 2]>,
    pub nvme_history: VecDeque<[f64; 2]>,
    pub active_profile: String,
    pub cooler_boost: bool,
    pub smart_boost: bool,
    pub battery_limit: u32,
    pub step_counter: usize,
    pub max_history: usize,
    pub tx: Sender<Result<String, String>>,
    pub rx: Receiver<Result<String, String>>,
}

impl TabHardware {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            telemetry: HardwareTelemetry::default(),
            cpu_history: VecDeque::with_capacity(60),
            gpu_history: VecDeque::with_capacity(60),
            nvme_history: VecDeque::with_capacity(60),
            active_profile: "balance".into(),
            cooler_boost: false,
            smart_boost: false,
            battery_limit: 80,
            step_counter: 0,
            max_history: 60,
            tx,
            rx,
        }
    }

    pub fn update_telemetry(&mut self) {
        self.telemetry = HardwareManager::read_telemetry();
        self.step_counter += 1;

        let x = self.step_counter as f64;
        let cpu_y = self.telemetry.cpu_temp as f64;
        let gpu_y = self.telemetry.gpu_temp as f64;
        let nvme_y = self.telemetry.nvme_temp as f64;

        if self.cpu_history.len() >= self.max_history {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back([x, cpu_y]);

        if self.gpu_history.len() >= self.max_history {
            self.gpu_history.pop_front();
        }
        self.gpu_history.push_back([x, gpu_y]);

        if self.nvme_history.len() >= self.max_history {
            self.nvme_history.pop_front();
        }
        self.nvme_history.push_back([x, nvme_y]);

        // Auto trigger smart boost if enabled and temp exceeds 80°C
        if self.smart_boost
            && (self.telemetry.cpu_temp >= 80.0 || self.telemetry.gpu_temp >= 80.0)
            && !self.cooler_boost
        {
            self.cooler_boost = true;
            let tx = self.tx.clone();
            thread::spawn(move || {
                let _ = tx.send(HardwareManager::set_cooler_boost(true));
            });
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        // Poll async hardware action completions
        while let Ok(res) = self.rx.try_recv() {
            match res {
                Ok(msg) => set_status(msg, GREEN_OK),
                Err(err) => set_status(err, RED_BRIGHT),
            }
        }

        ui.add_space(4.0);

        // 1. Top 4-Column Telemetry Tiles
        ui.columns(4, |cols| {
            let cpu_color = if self.telemetry.cpu_temp >= 80.0 {
                RED_BRIGHT
            } else if self.telemetry.cpu_temp >= 65.0 {
                AMBER_WARN
            } else {
                GREEN_OK
            };
            metric_tile(
                &mut cols[0],
                "CPU PACKAGE",
                &format!("{:.0}°C", self.telemetry.cpu_temp),
                &format!("{:.2} GHz • {}", self.telemetry.cpu_freq_ghz, self.telemetry.cpu_governor),
                cpu_color,
            );

            metric_tile(
                &mut cols[1],
                "GPU / PCH ZONE",
                &format!("{:.0}°C", self.telemetry.gpu_temp),
                if self.cooler_boost { "FAN: MAX BOOST" } else { "FAN: EC AUTO" },
                RED_PRIMARY,
            );

            metric_tile(
                &mut cols[2],
                "NVMe COMPOSITE",
                &format!("{:.0}°C", self.telemetry.nvme_temp),
                "PCIe Gen3 x4 SSD",
                CYAN_INFO,
            );

            let bat_color = if self.telemetry.battery_pct <= 20 {
                RED_BRIGHT
            } else {
                GREEN_OK
            };
            metric_tile(
                &mut cols[3],
                "BATTERY CELL",
                &format!("{}%", self.telemetry.battery_pct),
                &format!("{} (Cap {}%)", self.telemetry.battery_status, self.battery_limit),
                bat_color,
            );
        });

        ui.add_space(8.0);

        // 2. Realtime Thermal Waveform Plot Card
        let thermal_badge = if self.telemetry.cpu_temp >= 80.0 {
            ("🔥 HIGH THERMAL LOAD", RED_BRIGHT, RED_SUBTLE)
        } else {
            ("● NOMINAL THERMALS", GREEN_OK, GREEN_SUBTLE)
        };

        cyber_card(
            ui,
            "REALTIME THERMAL WAVEFORM",
            Some("60s ROLLING SENSOR MATRIX"),
            RED_PRIMARY,
            Some(thermal_badge),
            |ui| {
                let cpu_pts: Vec<[f64; 2]> = self.cpu_history.iter().copied().collect();
                let gpu_pts: Vec<[f64; 2]> = self.gpu_history.iter().copied().collect();
                let nvme_pts: Vec<[f64; 2]> = self.nvme_history.iter().copied().collect();

                let plot = Plot::new("thermal_plot")
                    .legend(Legend::default())
                    .height(165.0)
                    .include_y(25.0)
                    .include_y(95.0)
                    .show_axes([false, true])
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new("CPU Package (°C)", PlotPoints::new(cpu_pts))
                            .color(RED_BRIGHT)
                            .width(2.2),
                    );
                    plot_ui.line(
                        Line::new("GPU / Thermal Zone (°C)", PlotPoints::new(gpu_pts))
                            .color(AMBER_WARN)
                            .width(1.8),
                    );
                    plot_ui.line(
                        Line::new("NVMe SSD (°C)", PlotPoints::new(nvme_pts))
                            .color(CYAN_INFO)
                            .width(1.5),
                    );
                });
            },
        );

        ui.add_space(8.0);

        // 3. Power Profiles Deck
        let active_badge = format!("ACTIVE: {}", self.active_profile.to_uppercase());
        cyber_card(
            ui,
            "SYSTEM POWER PROFILES",
            Some("DISPLAY REFRESH + TLP + EC FAN DISPATCH"),
            RED_PRIMARY,
            Some((&active_badge, RED_BRIGHT, RED_SUBTLE)),
            |ui| {
                ui.columns(3, |cols| {
                    let profiles = [
                        ("extreme", "🔥 EXTREME PERFORMANCE", "144Hz • AC Full Power • Cooler Boost ON"),
                        ("balance", "⚖️ BALANCED MATRIX", "48Hz • Smart Auto Fans • Daily Driver"),
                        ("battery", "🔋 SUPER BATTERY SAVER", "48Hz • Silent Fans • Bluetooth Radio OFF"),
                    ];

                    for (idx, (id, title, desc)) in profiles.iter().enumerate() {
                        let is_selected = self.active_profile == *id;
                        let fill = if is_selected { RED_CRIMSON } else { BG_CARD };
                        let stroke = if is_selected {
                            egui::Stroke::new(1.5, RED_BRIGHT)
                        } else {
                            egui::Stroke::new(1.0, BORDER_DARK)
                        };

                        let btn_text = format!("{title}\n{desc}");
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text)
                                .strong()
                                .size(11.0)
                                .color(if is_selected { egui::Color32::WHITE } else { TEXT_MAIN }),
                        )
                        .fill(fill)
                        .stroke(stroke)
                        .min_size(egui::vec2(cols[idx].available_width(), 48.0));

                        if cols[idx].add(btn).clicked() {
                            self.active_profile = (*id).to_string();
                            self.cooler_boost = *id == "extreme";
                            set_status(format!("Dispatching {} profile...", id.to_uppercase()), CYAN_INFO);
                            let id_owned = (*id).to_string();
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::apply_power_profile(&id_owned));
                            });
                        }
                    }
                });
            },
        );

        ui.add_space(8.0);

        // 4. Dual Bottom Cards: Fan Controls & Battery Health Cap
        ui.columns(2, |cols| {
            // Left: Fan Controls
            let boost_badge = if self.cooler_boost {
                ("BOOST ACTIVE", RED_BRIGHT, RED_SUBTLE)
            } else {
                ("AUTO EC CURVE", TEXT_MUTED, BG_CARD)
            };

            cyber_card(
                &mut cols[0],
                "EC FAN MATRIX",
                Some("MSI 16G41"),
                RED_PRIMARY,
                Some(boost_badge),
                |ui| {
                    ui.horizontal(|ui| {
                        let boost_label = if self.cooler_boost {
                            "❄️ COOLER BOOST: ON (MAX RPM)"
                        } else {
                            "❄️ COOLER BOOST: OFF"
                        };
                        let btn_color = if self.cooler_boost { RED_PRIMARY } else { BG_CARD };
                        let btn = egui::Button::new(egui::RichText::new(boost_label).strong().size(11.5))
                            .fill(btn_color)
                            .min_size(egui::vec2(210.0, 34.0));

                        if ui.add(btn).clicked() {
                            self.cooler_boost = !self.cooler_boost;
                            let target = self.cooler_boost;
                            set_status(
                                format!("Setting Cooler Boost {}...", if target { "ON" } else { "OFF" }),
                                CYAN_INFO,
                            );
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::set_cooler_boost(target));
                            });
                        }

                        ui.checkbox(
                            &mut self.smart_boost,
                            egui::RichText::new("🤖 Smart (>80°C)").size(11.0).color(TEXT_MAIN),
                        )
                        .on_hover_text("Automatically engages Cooler Boost when CPU or GPU reaches 80°C");
                    });

                    ui.add_space(6.0);

                    ui.columns(2, |sub_cols| {
                        if sub_cols[0]
                            .add_sized(
                                [sub_cols[0].available_width(), 30.0],
                                egui::Button::new(egui::RichText::new("📈 Apply Custom Curve").size(11.0)),
                            )
                            .clicked()
                        {
                            set_status("Applying ISW custom curve (16G41)...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::apply_custom_fan_curve());
                            });
                        }

                        if sub_cols[1]
                            .add_sized(
                                [sub_cols[1].available_width(), 30.0],
                                egui::Button::new(egui::RichText::new("🔄 Reset BIOS Auto").size(11.0)),
                            )
                            .clicked()
                        {
                            self.cooler_boost = false;
                            set_status("Resetting fans to BIOS auto...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::reset_fans_auto());
                            });
                        }
                    });
                },
            );

            // Right: Battery Health Cap
            let cap_badge = format!("CAP: {}%", self.battery_limit);
            cyber_card(
                &mut cols[1],
                "BATTERY LONGEVITY CAP",
                Some("EC CHARGE LIMIT"),
                GREEN_OK,
                Some((&cap_badge, GREEN_OK, GREEN_SUBTLE)),
                |ui| {
                    ui.label(
                        egui::RichText::new("Limits AC charge ceiling in the Embedded Controller to reduce Li-ion cell wear:")
                            .size(10.0)
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(6.0);

                    ui.columns(3, |b_cols| {
                        let caps = [
                            (60u32, "🛡️ 60% CAP", "Max Lifespan"),
                            (80u32, "⚖️ 80% CAP", "Balanced Daily"),
                            (100u32, "⚡ 100% CAP", "Full Travel"),
                        ];

                        for (idx, (pct, title, sub)) in caps.iter().enumerate() {
                            let is_active = self.battery_limit == *pct;
                            let fill = if is_active { RED_CRIMSON } else { BG_CARD };
                            let stroke = if is_active {
                                egui::Stroke::new(1.5, RED_BRIGHT)
                            } else {
                                egui::Stroke::new(1.0, BORDER_DARK)
                            };

                            let btn = egui::Button::new(
                                egui::RichText::new(format!("{title}\n{sub}"))
                                    .strong()
                                    .size(10.5)
                                    .color(if is_active { egui::Color32::WHITE } else { TEXT_MAIN }),
                            )
                            .fill(fill)
                            .stroke(stroke)
                            .min_size(egui::vec2(b_cols[idx].available_width(), 44.0));

                            if b_cols[idx].add(btn).clicked() {
                                self.battery_limit = *pct;
                                let target_pct = *pct;
                                set_status(format!("Setting EC battery cap to {target_pct}%..."), CYAN_INFO);
                                let tx = self.tx.clone();
                                thread::spawn(move || {
                                    let _ = tx.send(HardwareManager::set_battery_limit(target_pct));
                                });
                            }
                        }
                    });
                },
            );
        });
    }
}
