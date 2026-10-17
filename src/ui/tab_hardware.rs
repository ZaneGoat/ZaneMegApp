use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::hardware::{CpuTimes, HardwareManager, HardwareTelemetry};
use crate::theme::*;

pub struct TabHardware {
    pub telemetry: HardwareTelemetry,
    pub prev_cpu_overall: CpuTimes,
    pub prev_cpu_cores: Vec<CpuTimes>,
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
        let (prev_cpu_overall, prev_cpu_cores) = HardwareManager::read_cpu_times();
        Self {
            telemetry: HardwareTelemetry::default(),
            prev_cpu_overall,
            prev_cpu_cores,
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
        self.telemetry = HardwareManager::read_telemetry(
            &mut self.prev_cpu_overall,
            &mut self.prev_cpu_cores,
        );
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
        while let Ok(res) = self.rx.try_recv() {
            match res {
                Ok(msg) => set_status(msg, GREEN_OK),
                Err(err) => set_status(err, RED_BRIGHT),
            }
        }

        // Visual Guide Strip
        legend_strip(
            ui,
            "Real-time temperatures, CPU core activity, display refresh rate, and fan cooling controls.",
            &[
                ("Optimal (<65°C)", GREEN_OK),
                ("Warm (65–79°C)", AMBER_WARN),
                ("High Heat (≥80°C)", RED_BRIGHT),
            ],
        );

        ui.add_space(6.0);

        // 1. Top 4-Column Radial Cockpit Arc Dials
        ui.columns(4, |cols| {
            let load_frac = self.telemetry.cpu_load_pct / 100.0;
            let load_color = if self.telemetry.cpu_load_pct >= 80.0 {
                RED_BRIGHT
            } else if self.telemetry.cpu_load_pct >= 45.0 {
                AMBER_WARN
            } else {
                CYAN_INFO
            };
            radial_dial(
                &mut cols[0],
                "CPU ACTIVITY",
                &format!("{:.0}%", self.telemetry.cpu_load_pct),
                &format!("{:.2} GHz • {}", self.telemetry.cpu_freq_ghz, self.telemetry.cpu_governor),
                load_frac,
                load_color,
            );

            let cpu_t_frac = ((self.telemetry.cpu_temp - 25.0) / 75.0).clamp(0.0, 1.0);
            let (cpu_color, cpu_desc) = if self.telemetry.cpu_temp >= 80.0 {
                (RED_BRIGHT, "High Thermal Load")
            } else if self.telemetry.cpu_temp >= 65.0 {
                (AMBER_WARN, "Warm / Active")
            } else {
                (GREEN_OK, "Cooling Normally")
            };
            radial_dial(
                &mut cols[1],
                "CPU TEMPERATURE",
                &format!("{:.0}°C", self.telemetry.cpu_temp),
                cpu_desc,
                cpu_t_frac,
                cpu_color,
            );

            let gpu_t_frac = ((self.telemetry.gpu_temp - 25.0) / 75.0).clamp(0.0, 1.0);
            radial_dial(
                &mut cols[2],
                "GPU & SSD TEMP",
                &format!("{:.0}°C", self.telemetry.gpu_temp),
                &format!("NVMe SSD: {:.0}°C", self.telemetry.nvme_temp),
                gpu_t_frac,
                RED_PRIMARY,
            );

            let bat_frac = self.telemetry.battery_pct as f32 / 100.0;
            let bat_color = if self.telemetry.battery_pct <= 20 {
                RED_BRIGHT
            } else {
                GREEN_OK
            };
            radial_dial(
                &mut cols[3],
                "BATTERY LEVEL",
                &format!("{}%", self.telemetry.battery_pct),
                &format!("{} (Limit {}%)", self.telemetry.battery_status, self.battery_limit),
                bat_frac,
                bat_color,
            );
        });

        ui.add_space(6.0);

        // 2. Power Profiles Selector Cards
        let active_badge = format!("CURRENT MODE: {}", self.active_profile.to_uppercase());
        cyber_card(
            ui,
            "1-CLICK SYSTEM POWER PROFILES",
            Some("Choose how your display, fans, and battery behave"),
            RED_PRIMARY,
            Some((&active_badge, RED_BRIGHT, RED_SUBTLE)),
            |ui| {
                ui.columns(3, |cols| {
                    if mode_selector_card(
                        &mut cols[0],
                        "🔥",
                        "EXTREME GAMING",
                        "🖥️ Screen: 144Hz Ultra Smooth",
                        "🌀 Fans: 100% Cooler Boost",
                        "⚡ Power: Maximum AC Speed",
                        RED_BRIGHT,
                        self.active_profile == "extreme",
                    ) {
                        self.active_profile = "extreme".into();
                        self.cooler_boost = true;
                        set_status("Applying EXTREME Gaming Profile...".into(), CYAN_INFO);
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(HardwareManager::apply_power_profile("extreme"));
                        });
                    }

                    if mode_selector_card(
                        &mut cols[1],
                        "⚖️",
                        "BALANCED DAILY",
                        "🖥️ Screen: 48Hz Power Saving",
                        "🌀 Fans: Automatic Smart Curve",
                        "⚡ Power: Balanced AC/Battery",
                        CYAN_INFO,
                        self.active_profile == "balance",
                    ) {
                        self.active_profile = "balance".into();
                        self.cooler_boost = false;
                        set_status("Applying BALANCED Daily Profile...".into(), CYAN_INFO);
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(HardwareManager::apply_power_profile("balance"));
                        });
                    }

                    if mode_selector_card(
                        &mut cols[2],
                        "🔋",
                        "SUPER BATTERY",
                        "🖥️ Screen: 48Hz Low Power",
                        "🌀 Fans: Quiet / Silent Mode",
                        "📡 Radio: Turns Bluetooth OFF",
                        GREEN_OK,
                        self.active_profile == "battery",
                    ) {
                        self.active_profile = "battery".into();
                        self.cooler_boost = false;
                        set_status("Applying SUPER BATTERY Profile...".into(), CYAN_INFO);
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            let _ = tx.send(HardwareManager::apply_power_profile("battery"));
                        });
                    }
                });
            },
        );

        ui.add_space(6.0);

        // 3. Dual Cards: Fan Cooling Controls & Battery Health Cap
        ui.columns(2, |cols| {
            // Left: Fan Cooling Matrix
            let boost_badge = if self.cooler_boost {
                ("MAX RPM ACTIVE", RED_BRIGHT, RED_SUBTLE)
            } else {
                ("AUTO COOLING", GREEN_OK, GREEN_SUBTLE)
            };

            cyber_card(
                &mut cols[0],
                "FAN COOLING CONTROLS",
                Some("Override or automate laptop fans"),
                RED_PRIMARY,
                Some(boost_badge),
                |ui| {
                    let boost_title = if self.cooler_boost {
                        "Cooler Boost is ON (100% Max Fan Speed)"
                    } else {
                        "Toggle Cooler Boost (100% Max Fan Speed)"
                    };
                    let boost_tag = if self.cooler_boost { ("ON", RED_BRIGHT) } else { ("OFF", TEXT_MUTED) };

                    if action_card_button(
                        ui,
                        "❄️",
                        boost_title,
                        "Forces both CPU and GPU fans to spin at maximum RPM immediately",
                        Some(boost_tag),
                        RED_BRIGHT,
                        self.cooler_boost,
                    ) {
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

                    ui.add_space(4.0);

                    let smart_tag = if self.smart_boost {
                        ("ENABLED", GREEN_OK)
                    } else {
                        ("DISABLED", TEXT_DIM)
                    };
                    if action_card_button(
                        ui,
                        "🤖",
                        "Auto Smart Thermostat (80°C Trigger)",
                        "Automatically turns ON Cooler Boost if CPU or GPU reaches 80°C",
                        Some(smart_tag),
                        CYAN_INFO,
                        self.smart_boost,
                    ) {
                        self.smart_boost = !self.smart_boost;
                        set_status(
                            format!(
                                "Smart Thermostat {}",
                                if self.smart_boost { "ENABLED" } else { "DISABLED" }
                            ),
                            GREEN_OK,
                        );
                    }

                    ui.add_space(4.0);

                    ui.columns(2, |sub_cols| {
                        if action_card_button(
                            &mut sub_cols[0],
                            "📈",
                            "Custom Fan Curve",
                            "Load tuned MSI 16G41 table",
                            None,
                            RED_PRIMARY,
                            false,
                        ) {
                            set_status("Applying ISW custom curve (16G41)...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::apply_custom_fan_curve());
                            });
                        }

                        if action_card_button(
                            &mut sub_cols[1],
                            "🔄",
                            "Reset BIOS Auto",
                            "Return fans to default curve",
                            None,
                            GREEN_OK,
                            false,
                        ) {
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
            let cap_badge = format!("CHARGE CEILING: {}%", self.battery_limit);
            cyber_card(
                &mut cols[1],
                "BATTERY LIFESPAN PROTECTION",
                Some("Stop charging early to prevent battery wear"),
                GREEN_OK,
                Some((&cap_badge, GREEN_OK, GREEN_SUBTLE)),
                |ui| {
                    ui.columns(3, |b_cols| {
                        if mode_selector_card(
                            &mut b_cols[0],
                            "🛡️",
                            "60% MAX LIFE",
                            "• Best if always plugged in",
                            "• Minimizes cell voltage stress",
                            "• Doubles battery lifespan",
                            GREEN_OK,
                            self.battery_limit == 60,
                        ) {
                            self.battery_limit = 60;
                            set_status("Setting battery charge limit to 60%...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::set_battery_limit(60));
                            });
                        }

                        if mode_selector_card(
                            &mut b_cols[1],
                            "⚖️",
                            "80% BALANCED",
                            "• Ideal for mixed daily use",
                            "• Prevents 100% heat wear",
                            "• Good portable runtime",
                            CYAN_INFO,
                            self.battery_limit == 80,
                        ) {
                            self.battery_limit = 80;
                            set_status("Setting battery charge limit to 80%...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::set_battery_limit(80));
                            });
                        }

                        if mode_selector_card(
                            &mut b_cols[2],
                            "⚡",
                            "100% FULL TRIP",
                            "• Charges battery to 100%",
                            "• Maximum unplugged hours",
                            "• Use before travel",
                            AMBER_WARN,
                            self.battery_limit == 100,
                        ) {
                            self.battery_limit = 100;
                            set_status("Setting battery charge limit to 100%...".into(), CYAN_INFO);
                            let tx = self.tx.clone();
                            thread::spawn(move || {
                                let _ = tx.send(HardwareManager::set_battery_limit(100));
                            });
                        }
                    });

                    ui.add_space(6.0);
                    egui::Frame::new()
                        .fill(BG_VOID)
                        .corner_radius(6)
                        .inner_margin(8)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(
                                    "ℹ️ Tip: Keeping a lithium-ion battery at 60%–80% while on AC power prevents chemical degradation and swelling.",
                                )
                                .size(9.5)
                                .color(TEXT_MUTED),
                            );
                        });
                },
            );
        });

        ui.add_space(6.0);

        // 4. Realtime Thermal Waveform & 12-Core Activity
        ui.columns(2, |cols| {
            cyber_card(
                &mut cols[0],
                "TEMPERATURE HISTORY (LAST 60 SECONDS)",
                Some("Live sensor graph"),
                RED_PRIMARY,
                None,
                |ui| {
                    let cpu_pts: Vec<[f64; 2]> = self.cpu_history.iter().copied().collect();
                    let gpu_pts: Vec<[f64; 2]> = self.gpu_history.iter().copied().collect();
                    let nvme_pts: Vec<[f64; 2]> = self.nvme_history.iter().copied().collect();

                    let plot = Plot::new("thermal_plot")
                        .legend(Legend::default())
                        .height(135.0)
                        .include_y(25.0)
                        .include_y(95.0)
                        .show_axes([false, true])
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false);

                    plot.show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("CPU (°C)", PlotPoints::new(cpu_pts))
                                .color(RED_BRIGHT)
                                .width(2.2),
                        );
                        plot_ui.line(
                            Line::new("GPU/Zone (°C)", PlotPoints::new(gpu_pts))
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

            cyber_card(
                &mut cols[1],
                "12-CORE PROCESSOR ACTIVITY",
                Some("Per-thread load on Intel Core i5-10500H"),
                CYAN_INFO,
                None,
                |ui| {
                    let n = self.telemetry.core_loads.len().min(12);
                    if n >= 6 {
                        ui.columns(3, |c| {
                            for i in 0..3 {
                                mini_core_bar(&mut c[i], i, self.telemetry.core_loads[i]);
                            }
                        });
                        ui.add_space(3.0);
                        ui.columns(3, |c| {
                            for i in 3..6 {
                                mini_core_bar(&mut c[i - 3], i, self.telemetry.core_loads[i]);
                            }
                        });
                        if n >= 12 {
                            ui.add_space(3.0);
                            ui.columns(3, |c| {
                                for i in 6..9 {
                                    mini_core_bar(&mut c[i - 6], i, self.telemetry.core_loads[i]);
                                }
                            });
                            ui.add_space(3.0);
                            ui.columns(3, |c| {
                                for i in 9..12 {
                                    mini_core_bar(&mut c[i - 9], i, self.telemetry.core_loads[i]);
                                }
                            });
                        }
                    }
                },
            );
        });
    }
}
