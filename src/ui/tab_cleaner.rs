use eframe::egui;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::core::cleaner::{CleanerManager, SystemResources, MODULES};
use crate::theme::*;

pub struct TabCleaner {
    pub selected_ids: HashSet<String>,
    pub resources: SystemResources,
    pub top_procs: Vec<(u32, String, f32)>,
    pub last_reclaimed_banner: Option<String>,
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
            resources: CleanerManager::read_system_resources(),
            top_procs: CleanerManager::read_top_processes(5),
            last_reclaimed_banner: None,
            logs: vec![
                "[*] System Cleaner Matrix online.".into(),
                "[*] Select target modules, snipe heavy processes, or trigger Quick RAM Boost.".into(),
            ],
            is_running: false,
            tx,
            rx,
        }
    }

    pub fn update_resources(&mut self) {
        self.resources = CleanerManager::read_system_resources();
        self.top_procs = CleanerManager::read_top_processes(5);
    }

    pub fn poll_logs(&mut self) {
        while let Ok(line) = self.rx.try_recv() {
            if line.contains("RECLAIMED //") {
                self.last_reclaimed_banner = Some(line.replace("[✓] ", ""));
                self.update_resources();
            }
            if line.contains("CLEANUP PROTOCOL COMPLETED") {
                self.is_running = false;
                self.update_resources();
            }
            self.logs.push(line);
            if self.logs.len() > 1000 {
                self.logs.remove(0);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, set_status: &mut dyn FnMut(String, egui::Color32)) {
        self.poll_logs();
        ui.add_space(4.0);

        // 1. Live System Resource Gauges Deck
        let badge = if let Some(ref banner) = self.last_reclaimed_banner {
            Some((banner.as_str(), GREEN_OK, GREEN_SUBTLE))
        } else {
            Some(("LIVE PROCFS & VFS TELEMETRY", CYAN_INFO, CYAN_SUBTLE))
        };

        cyber_card(
            ui,
            "LIVE MEMORY & STORAGE GAUGES",
            Some("REALTIME RESOURCE UTILIZATION"),
            RED_PRIMARY,
            badge,
            |ui| {
                ui.columns(3, |cols| {
                    // Gauge 1: Physical RAM
                    let ram_frac = self.resources.ram_percent / 100.0;
                    let ram_used_str = format!(
                        "{:.2} / {:.2} GB",
                        self.resources.ram_used_gb, self.resources.ram_total_gb
                    );
                    let ram_cache_str = format!("Cache: {:.0} MB", self.resources.ram_cached_mb);
                    resource_gauge_card(
                        &mut cols[0],
                        "🧠 PHYSICAL RAM",
                        &ram_used_str,
                        &ram_cache_str,
                        ram_frac,
                    );

                    // Gauge 2: Swap Memory
                    let swap_frac = self.resources.swap_percent / 100.0;
                    let swap_used_str = format!(
                        "{:.2} / {:.2} GB",
                        self.resources.swap_used_gb, self.resources.swap_total_gb
                    );
                    let swap_detail = if self.resources.swap_used_gb < 0.05 {
                        "Clean"
                    } else {
                        "Active Paging"
                    };
                    resource_gauge_card(
                        &mut cols[1],
                        "🔄 SWAP SPACE",
                        &swap_used_str,
                        swap_detail,
                        swap_frac,
                    );

                    // Gauge 3: Root NVMe Disk
                    let disk_frac = self.resources.disk_percent / 100.0;
                    let disk_used_str = format!(
                        "{:.1} / {:.1} GB",
                        self.resources.disk_used_gb, self.resources.disk_total_gb
                    );
                    let disk_free_str = format!("Free: {:.1} GB", self.resources.disk_free_gb);
                    resource_gauge_card(
                        &mut cols[2],
                        "💾 ROOT NVMe (/)",
                        &disk_used_str,
                        &disk_free_str,
                        disk_frac,
                    );
                });
            },
        );

        ui.add_space(6.0);

        // 2. Main Split View: Module Matrix (Left) + Process Sniper & Console (Right)
        ui.horizontal_top(|ui| {
            // Left Sidebar: Cleanup Modules
            egui::Frame::new()
                .fill(BG_PANEL)
                .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(310.0);

                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 15.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, RED_PRIMARY);
                        ui.label(
                            egui::RichText::new("OPTIMIZATION MODULES")
                                .strong()
                                .size(12.0)
                                .color(TEXT_MAIN),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            status_pill(
                                ui,
                                &format!("{}/{} SELECTED", self.selected_ids.len(), MODULES.len()),
                                RED_BRIGHT,
                                RED_SUBTLE,
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Preset Selector Row
                    ui.horizontal(|ui| {
                        if ui.small_button("✔ All").clicked() {
                            for m in MODULES {
                                self.selected_ids.insert(m.id.to_string());
                            }
                        }
                        if ui.small_button("✖ None").clicked() {
                            self.selected_ids.clear();
                        }
                        if ui.small_button("🛡 Safe Default").clicked() {
                            self.selected_ids.clear();
                            for m in MODULES {
                                if m.default_checked {
                                    self.selected_ids.insert(m.id.to_string());
                                }
                            }
                        }
                        if ui.small_button("🧠 RAM Only").clicked() {
                            self.selected_ids.clear();
                            self.selected_ids.insert("smart_ram".into());
                            self.selected_ids.insert("kernel_cache".into());
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_salt("cleaner_modules_scroll")
                        .max_height(310.0)
                        .show(ui, |ui| {
                            for m in MODULES {
                                let is_checked = self.selected_ids.contains(m.id);
                                let mut checked = is_checked;
                                let row_bg = if is_checked { BG_CARD } else { BG_PANEL };
                                let text_color = if is_checked { TEXT_MAIN } else { TEXT_MUTED };

                                egui::Frame::new()
                                    .fill(row_bg)
                                    .corner_radius(5)
                                    .inner_margin(egui::Margin::symmetric(8, 4))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui
                                                .checkbox(
                                                    &mut checked,
                                                    egui::RichText::new(m.name).size(11.0).color(text_color),
                                                )
                                                .on_hover_text(m.description)
                                                .changed()
                                            {
                                                if checked {
                                                    self.selected_ids.insert(m.id.to_string());
                                                } else {
                                                    self.selected_ids.remove(m.id);
                                                }
                                            }

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if m.requires_root {
                                                    ui.label(
                                                        egui::RichText::new("ROOT")
                                                            .monospace()
                                                            .size(8.5)
                                                            .color(AMBER_WARN),
                                                    );
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(m.category)
                                                            .monospace()
                                                            .size(8.5)
                                                            .color(TEXT_DIM),
                                                    );
                                                }
                                            });
                                        });
                                    });
                            }
                        });

                    ui.add_space(8.0);

                    // Primary Execute Button
                    let exec_btn = egui::Button::new(
                        egui::RichText::new(if self.is_running {
                            "⏳ EXECUTING PROTOCOL..."
                        } else {
                            "🚀 EXECUTE SELECTED CLEANUP"
                        })
                        .strong()
                        .size(12.5)
                        .color(egui::Color32::WHITE),
                    )
                    .fill(if self.is_running { BG_CARD } else { RED_PRIMARY })
                    .min_size(egui::vec2(ui.available_width(), 38.0));

                    if ui.add_enabled(!self.is_running && !self.selected_ids.is_empty(), exec_btn).clicked() {
                        self.is_running = true;
                        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            CleanerManager::execute_cleanup(&ids, tx);
                        });
                        set_status("Executing system cleanup protocol...".into(), GREEN_OK);
                    }

                    ui.add_space(4.0);

                    // 1-Click Quick RAM Boost
                    let quick_ram_btn = egui::Button::new(
                        egui::RichText::new("⚡ 1-CLICK QUICK RAM BOOST (NO ROOT)")
                            .strong()
                            .size(11.0)
                            .color(CYAN_INFO),
                    )
                    .fill(CYAN_SUBTLE)
                    .min_size(egui::vec2(ui.available_width(), 30.0));

                    if ui.add_enabled(!self.is_running, quick_ram_btn).clicked() {
                        self.is_running = true;
                        let ids = vec!["smart_ram".to_string(), "thumbnail_cache".to_string()];
                        let tx = self.tx.clone();
                        thread::spawn(move || {
                            CleanerManager::execute_cleanup(&ids, tx);
                        });
                        set_status("Running 1-Click Smart RAM Boost...".into(), CYAN_INFO);
                    }
                });

            ui.add_space(6.0);

            // Right Pane: Top RAM Hogs Process Sniper + Cyber Console Log
            ui.vertical(|ui| {
                // Top Memory Hogs Live Sniper Card
                cyber_card(
                    ui,
                    "🎯 LIVE TOP RAM HOGS // PROCESS SNIPER",
                    Some("REALTIME /PROC RSS SCANNER"),
                    AMBER_WARN,
                    Some(("SIGTERM DISPATCH", AMBER_WARN, AMBER_SUBTLE)),
                    |ui| {
                        let procs_snapshot = self.top_procs.clone();
                        let mut sniped_action: Option<(u32, String)> = None;

                        for (pid, name, rss_mb) in procs_snapshot {
                            egui::Frame::new()
                                .fill(BG_CARD)
                                .corner_radius(5)
                                .inner_margin(egui::Margin::symmetric(10, 4))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("PID {pid:<6}"))
                                                .monospace()
                                                .size(10.0)
                                                .color(TEXT_DIM),
                                        );
                                        ui.label(
                                            egui::RichText::new(&name)
                                                .strong()
                                                .size(11.0)
                                                .color(TEXT_MAIN),
                                        );

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("🎯 SNIPE")
                                                            .strong()
                                                            .size(9.5)
                                                            .color(RED_BRIGHT),
                                                    )
                                                    .fill(RED_SUBTLE)
                                                    .min_size(egui::vec2(64.0, 20.0)),
                                                )
                                                .on_hover_text(format!("Send SIGTERM to {name} (PID {pid})"))
                                                .clicked()
                                            {
                                                sniped_action = Some((pid, name.clone()));
                                            }

                                            ui.add_space(6.0);
                                            let mem_color = if rss_mb >= 800.0 {
                                                RED_BRIGHT
                                            } else if rss_mb >= 300.0 {
                                                AMBER_WARN
                                            } else {
                                                CYAN_INFO
                                            };
                                            ui.label(
                                                egui::RichText::new(format!("{rss_mb:.0} MB"))
                                                    .monospace()
                                                    .strong()
                                                    .size(10.5)
                                                    .color(mem_color),
                                            );
                                        });
                                    });
                                });
                        }

                        if let Some((pid, name)) = sniped_action {
                            CleanerManager::snipe_process(pid, &name, &self.tx);
                            self.update_resources();
                            set_status(format!("Sent SIGTERM to {name} (PID {pid})"), AMBER_WARN);
                        }
                    },
                );

                ui.add_space(6.0);

                // Execution Console
                cyber_console(
                    ui,
                    "cleaner_console_scroll",
                    "SYSTEM CLEANER // REALTIME EXECUTION MATRIX",
                    &mut self.logs,
                    220.0,
                );
            });
        });
    }
}
